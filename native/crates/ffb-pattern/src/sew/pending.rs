use std::cell::RefCell;

use crate::{Panel, Point};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Split { panel: u32, inner: Option<u32>, segment: usize, cuts: Vec<f64>, points: usize },
    Reproject { panel: u32, old: Vec<Point>, points: usize },
    Reverse { panel: u32, points: usize },
}

thread_local! {
    static QUEUE: RefCell<Vec<Op>> = const { RefCell::new(Vec::new()) };
}

fn push(op: Op) {
    QUEUE.with(|q| q.borrow_mut().push(op));
}

pub fn note_split(panel: &Panel, inner: Option<u32>, segment: usize, cuts: &[f64]) {
    let points = panel.path(inner).map_or(0, |(p, _)| p.len());
    push(Op::Split { panel: panel.id, inner, segment, cuts: cuts.to_vec(), points });
}

pub fn note_reproject(panel: &Panel, old: &[Point]) {
    push(Op::Reproject { panel: panel.id, old: old.to_vec(), points: panel.outline.len() });
}

pub fn note_reverse(panel: &Panel) {
    push(Op::Reverse { panel: panel.id, points: panel.outline.len() });
}

pub fn take_pending() -> Vec<Op> {
    QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}
