use std::cmp::Ordering;

use super::{Place, VehicleStore};

const NO_INDEX: u32 = u32::MAX;
const MOVEMENT_BIT: u64 = 1 << 63;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub key: u64,
    pub s: f64,
    pub slot: u32,
}

#[derive(Default)]
pub struct Occupancy {
    entries: Vec<Entry>,
    index: Vec<u32>,
    pending: Vec<Entry>,
}

pub fn place_key(place: Place) -> u64 {
    match place {
        Place::Link { link, lane } => (u64::from(link) << 8) | u64::from(lane),
        Place::Movement { node, movement, .. } => {
            MOVEMENT_BIT | (u64::from(node) << 16) | u64::from(movement)
        }
    }
}

fn order(a: &Entry, b: &Entry) -> Ordering {
    a.key
        .cmp(&b.key)
        .then_with(|| b.s.total_cmp(&a.s))
        .then_with(|| a.slot.cmp(&b.slot))
}

impl Occupancy {
    pub fn rebuild(&mut self, vehicles: &VehicleStore) {
        self.entries.clear();
        self.pending.clear();
        for slot in vehicles.live_slots() {
            self.entries.push(entry_of(vehicles, slot));
        }
        self.entries.sort_unstable_by(order);
        self.index.clear();
        self.index.resize(vehicles.slot_count(), NO_INDEX);
        for (i, entry) in self.entries.iter().enumerate() {
            self.index[entry.slot as usize] = i as u32;
        }
    }

    pub fn push_pending(&mut self, entry: Entry) {
        self.pending.push(entry);
    }

    pub fn forget(&mut self, slot: u32) {
        if let Some(index) = self.index.get_mut(slot as usize) {
            *index = NO_INDEX;
        }
    }

    pub fn index_of(&self, slot: u32) -> Option<usize> {
        let index = *self.index.get(slot as usize)?;
        (index != NO_INDEX).then_some(index as usize)
    }

    pub fn same_place_leader(&self, slot: u32, key: u64) -> Option<Entry> {
        let Some(index) = self.index_of(slot) else {
            return self.last_sorted(key);
        };
        let previous = self.entries.get(index.checked_sub(1)?)?;
        (previous.key == key).then_some(*previous)
    }

    pub fn last_on(&self, key: u64) -> Option<(u32, f64)> {
        let sorted = self.last_sorted(key);
        let pending = self
            .pending
            .iter()
            .filter(|entry| entry.key == key)
            .min_by(|a, b| a.s.total_cmp(&b.s).then_with(|| a.slot.cmp(&b.slot)));
        let best = match (sorted, pending) {
            (Some(a), Some(b)) if b.s < a.s => *b,
            (Some(a), _) => a,
            (None, Some(b)) => *b,
            (None, None) => return None,
        };
        Some((best.slot, best.s))
    }

    fn last_sorted(&self, key: u64) -> Option<Entry> {
        let start = self.entries.partition_point(|entry| entry.key < key);
        let end = self.entries.partition_point(|entry| entry.key <= key);
        (start..end)
            .rev()
            .find(|&i| self.index_of(self.entries[i].slot) == Some(i))
            .map(|i| self.entries[i])
    }
}

pub fn entry_of(vehicles: &VehicleStore, slot: u32) -> Entry {
    let index = slot as usize;
    Entry {
        key: place_key(vehicles.place[index]),
        s: vehicles.s[index],
        slot,
    }
}
