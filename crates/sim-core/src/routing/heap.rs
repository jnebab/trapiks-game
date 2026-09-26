use std::cmp::Ordering;

use crate::network::LinkId;

#[derive(Clone, Copy, Debug)]
pub struct HeapItem {
    pub f: f64,
    pub g: f64,
    pub link: LinkId,
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .to_bits()
            .cmp(&self.f.to_bits())
            .then_with(|| self.g.to_bits().cmp(&other.g.to_bits()))
            .then_with(|| other.link.cmp(&self.link))
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for HeapItem {}
