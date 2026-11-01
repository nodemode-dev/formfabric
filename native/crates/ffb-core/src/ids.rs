#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IdGen {
    next: u32,
}

impl IdGen {
    pub fn starting_after(used: impl IntoIterator<Item = u32>) -> IdGen {
        IdGen { next: used.into_iter().max().map_or(1, |m| m + 1) }
    }

    pub fn next_id(&mut self) -> u32 {
        self.next = self.next.max(1);
        let id = self.next;
        self.next += 1;
        id
    }
}
