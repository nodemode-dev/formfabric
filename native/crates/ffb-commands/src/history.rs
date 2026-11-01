use ffb_core::Units;
use ffb_pattern::{Garment, Panel, Pin};

pub const DEFAULT_STEPS: usize = 200;
pub const MAX_STEPS: usize = 10_000;
const MAX_BYTES: usize = 512 * 1024 * 1024;

fn edit_bytes(edit: &Edit) -> usize {
    match edit {
        Edit::Units { .. } => 64,
        Edit::Garment { before, after, .. } => before.approx_bytes() + after.approx_bytes(),
        Edit::Bodies { changes, .. } => 65_536 * changes.len().max(1),
        Edit::Panel { before, after, .. } => [before, after].iter().filter_map(|p| p.as_ref()).map(|p| serde_json::to_vec(p).map_or(0, |b| b.len())).sum(),
        Edit::Many { edits, .. } => edits.iter().map(edit_bytes).sum(),
        Edit::Pins { before, after, .. } => (before.len() + after.len()) * 64,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Units { from: Units, to: Units },
    Garment { label: String, before: Box<Garment>, after: Box<Garment> },
    Bodies { label: String, changes: Vec<ffb_body::BodyChange> },
    Panel { label: String, index: usize, before: Option<Box<Panel>>, after: Option<Box<Panel>> },
    Many { label: String, edits: Vec<Edit> },
    Pins { label: String, before: Vec<Pin>, after: Vec<Pin> },
}

impl Edit {
    pub fn label(&self) -> String {
        match self {
            Edit::Units { .. } => "Units".into(),
            Edit::Garment { label, .. } | Edit::Panel { label, .. } | Edit::Many { label, .. } | Edit::Bodies { label, .. } | Edit::Pins { label, .. } => label.clone(),
        }
    }

    fn apply(&self, garment: &mut Garment, forward: bool) -> Result<(), String> {
        match self {
            Edit::Units { from, to } => garment.units = if forward { *to } else { *from },
            Edit::Bodies { .. } => {}
            Edit::Pins { before, after, .. } => garment.pins = (if forward { after } else { before }).clone(),
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
        Edit::Units { .. } | Edit::Bodies { .. } | Edit::Pins { .. } => false,
    }
}

fn in_place(edit: &Edit) -> bool {
    match edit {
        Edit::Panel { before, after, .. } => before.is_some() && after.is_some(),
        Edit::Many { edits, .. } => edits.iter().all(in_place),
        Edit::Garment { .. } | Edit::Bodies { .. } => false,
        Edit::Units { .. } | Edit::Pins { .. } => true,
    }
}

pub type BodySteps = Vec<(Vec<ffb_body::BodyChange>, bool)>;

fn collect_bodies(edit: &Edit, forward: bool, out: &mut BodySteps) {
    match edit {
        Edit::Bodies { changes, .. } => out.push((changes.clone(), forward)),
        Edit::Many { edits, .. } => {
            if forward {
                edits.iter().for_each(|e| collect_bodies(e, true, out));
            } else {
                edits.iter().rev().for_each(|e| collect_bodies(e, false, out));
            }
        }
        _ => {}
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
    sizes: Vec<usize>,
    cursor: usize,
    limit: usize,
    group: Option<(String, Vec<Edit>)>,
    bodies_out: BodySteps,
}

impl Default for History {
    fn default() -> Self {
        History { steps: Vec::new(), sizes: Vec::new(), cursor: 0, limit: DEFAULT_STEPS, group: None, bodies_out: Vec::new() }
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
        self.sizes.truncate(self.cursor);
        self.sizes.push(edit_bytes(&edit));
        self.steps.push(edit);
        let mut total: usize = self.sizes.iter().sum();
        while self.steps.len() > 1 && (self.steps.len() > self.limit || total > MAX_BYTES) {
            total = total.saturating_sub(self.sizes.remove(0));
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
        } else {
            let first = std::mem::replace(slot, Edit::Many { label: String::new(), edits: Vec::new() });
            let label = first.label();
            let mut edits = vec![first];
            absorb(&mut edits, edit, false);
            *slot = Edit::Many { label, edits };
        }
        self.sizes[self.cursor - 1] = edit_bytes(&self.steps[self.cursor - 1]);
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
        self.sizes.drain(..extra);
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
        collect_bodies(edit, false, &mut self.bodies_out);
        self.cursor -= 1;
        Ok(Some(edit.label()))
    }

    pub fn redo(&mut self, garment: &mut Garment) -> Result<Option<String>, String> {
        self.end_group();
        let Some(edit) = self.steps.get(self.cursor) else { return Ok(None) };
        edit.apply(garment, true)?;
        collect_bodies(edit, true, &mut self.bodies_out);
        self.cursor += 1;
        Ok(Some(edit.label()))
    }

    pub fn take_bodies(&mut self) -> BodySteps {
        std::mem::take(&mut self.bodies_out)
    }
}
