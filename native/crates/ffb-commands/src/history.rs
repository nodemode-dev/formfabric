use ffb_core::Units;
use ffb_pattern::{Garment, Panel};

pub const DEFAULT_STEPS: usize = 200;
pub const MAX_STEPS: usize = 10_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Units { from: Units, to: Units },
    Garment { label: String, before: Box<Garment>, after: Box<Garment> },
    Panel { label: String, index: usize, before: Option<Box<Panel>>, after: Option<Box<Panel>> },
    Many { label: String, edits: Vec<Edit> },
}

impl Edit {
    pub fn label(&self) -> String {
        match self {
            Edit::Units { .. } => "Units".into(),
            Edit::Garment { label, .. } | Edit::Panel { label, .. } | Edit::Many { label, .. } => label.clone(),
        }
    }

    fn apply(&self, garment: &mut Garment, forward: bool) -> Result<(), String> {
        match self {
            Edit::Units { from, to } => garment.units = if forward { *to } else { *from },
            Edit::Garment { before, after, .. } => *garment = (**if forward { after } else { before }).clone(),
            Edit::Panel { index, before, after, .. } => {
                let (from, to) = if forward { (before, after) } else { (after, before) };
                match (from, to) {
                    (None, Some(p)) if *index <= garment.panels.len() => garment.panels.insert(*index, (**p).clone()),
                    (Some(_), None) if *index < garment.panels.len() => {
                        garment.panels.remove(*index);
                    }
                    (Some(_), Some(p)) if *index < garment.panels.len() => garment.panels[*index] = (**p).clone(),
                    (None, None) => {}
                    _ => return Err("the edited panel is no longer there".into()),
                }
            }
            Edit::Many { edits, .. } => {
                if forward {
                    for e in edits {
                        e.apply(garment, true)?;
                    }
                } else {
                    for e in edits.iter().rev() {
                        e.apply(garment, false)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn touches(edit: &Edit, at: usize) -> bool {
    match edit {
        Edit::Panel { index, .. } => *index == at,
        Edit::Many { edits, .. } => edits.iter().any(|e| touches(e, at)),
        Edit::Garment { .. } => true,
        Edit::Units { .. } => false,
    }
}

fn in_place(edit: &Edit) -> bool {
    match edit {
        Edit::Panel { before, after, .. } => before.is_some() && after.is_some(),
        Edit::Many { edits, .. } => edits.iter().all(in_place),
        Edit::Garment { .. } => false,
        Edit::Units { .. } => true,
    }
}

fn absorb(edits: &mut Vec<Edit>, edit: Edit, front: bool) {
    if !edits.iter().all(in_place) {
        edits.push(edit);
        return;
    }
    if let Edit::Panel { index, before: Some(_), after: Some(after), .. } = &edit {
        for e in edits.iter_mut() {
            if let Edit::Panel { index: i, after: last, .. } = e {
                if i == index && last.is_some() {
                    *last = Some(after.clone());
                    return;
                }
            }
        }
        if front && !edits.iter().any(|e| touches(e, *index)) {
            edits.insert(0, edit);
            return;
        }
    }
    edits.push(edit);
}

#[derive(Debug)]
pub struct History {
    steps: Vec<Edit>,
    cursor: usize,
    limit: usize,
    group: Option<(String, Vec<Edit>)>,
}

impl Default for History {
    fn default() -> Self {
        History { steps: Vec::new(), cursor: 0, limit: DEFAULT_STEPS, group: None }
    }
}

impl History {
    pub fn record(&mut self, edit: Edit) {
        if let Some((_, edits)) = &mut self.group {
            match (edits.last_mut(), edit) {
                (Some(Edit::Panel { index: i, after: last, .. }), Edit::Panel { index, before: Some(_), after: Some(after), .. }) if *i == index && last.is_some() => *last = Some(after),
                (_, edit) => edits.push(edit),
            }
            return;
        }
        self.steps.truncate(self.cursor);
        self.steps.push(edit);
        if self.steps.len() > self.limit {
            self.steps.remove(0);
        }
        self.cursor = self.steps.len();
    }

    pub fn amend_last(&mut self, edit: Edit) {
        if let Some((_, edits)) = &mut self.group {
            absorb(edits, edit, true);
            return;
        }
        if self.cursor == 0 {
            self.record(edit);
            return;
        }
        let slot = &mut self.steps[self.cursor - 1];
        if let Edit::Many { edits, .. } = slot {
            absorb(edits, edit, false);
            return;
        }
        let first = std::mem::replace(slot, Edit::Many { label: String::new(), edits: Vec::new() });
        let label = first.label();
        let mut edits = vec![first];
        absorb(&mut edits, edit, false);
        *slot = Edit::Many { label, edits };
    }

    pub fn begin_group(&mut self, label: &str) {
        self.end_group();
        self.group = Some((label.to_string(), Vec::new()));
    }

    pub fn end_group(&mut self) {
        let Some((label, mut edits)) = self.group.take() else { return };
        match edits.len() {
            0 => {}
            1 => self.record(edits.remove(0)),
            _ => self.record(Edit::Many { label, edits }),
        }
    }

    pub fn grouping(&self) -> bool {
        self.group.is_some()
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.steps.len()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.clamp(1, MAX_STEPS);
        let extra = self.steps.len().saturating_sub(self.limit);
        self.steps.drain(..extra);
        self.cursor = self.cursor.saturating_sub(extra);
    }

    pub fn labels(&self) -> Vec<(String, bool)> {
        self.steps.iter().enumerate().map(|(i, e)| (e.label(), i < self.cursor)).collect()
    }

    pub fn undo(&mut self, garment: &mut Garment) -> Result<Option<String>, String> {
        self.end_group();
        if self.cursor == 0 {
            return Ok(None);
        }
        let edit = &self.steps[self.cursor - 1];
        edit.apply(garment, false)?;
        self.cursor -= 1;
        Ok(Some(edit.label()))
    }

    pub fn redo(&mut self, garment: &mut Garment) -> Result<Option<String>, String> {
        self.end_group();
        let Some(edit) = self.steps.get(self.cursor) else { return Ok(None) };
        edit.apply(garment, true)?;
        self.cursor += 1;
        Ok(Some(edit.label()))
    }
}
