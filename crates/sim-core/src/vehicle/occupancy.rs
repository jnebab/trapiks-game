use std::cmp::Ordering;

use crate::network::LinkId;

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
    link_first: Vec<u32>,
    node_first: Vec<u32>,
}

pub fn place_key(place: Place) -> u64 {
    match place {
        Place::Link { link, lane } => link_key(link, lane),
        Place::Movement { node, movement, .. } => movement_key(node, movement),
    }
}

pub fn link_key(link: LinkId, lane: u8) -> u64 {
    (u64::from(link) << 8) | u64::from(lane)
}

pub fn movement_key(node: u32, movement: u16) -> u64 {
    MOVEMENT_BIT | (u64::from(node) << 16) | u64::from(movement)
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
        let links_end = self.entries.partition_point(|e| e.key & MOVEMENT_BIT == 0);
        let (links, movements) = self.entries.split_at(links_end);
        fill_offsets(&mut self.link_first, links, 0);
        fill_offsets(&mut self.node_first, movements, links_end);
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

    pub fn slots_on(&self, key: u64) -> impl Iterator<Item = u32> + '_ {
        let pending = self.pending.iter().filter(move |entry| entry.key == key);
        self.range(key)
            .iter()
            .chain(pending)
            .map(|entry| entry.slot)
    }

    pub fn range(&self, key: u64) -> &[Entry] {
        &self.entries[self.bounds(key)]
    }

    pub fn count(&self, key: u64) -> usize {
        let pending = self.pending.iter().filter(|entry| entry.key == key).count();
        self.range(key).len() + pending
    }

    pub fn ahead(&self, slot: u32, key: u64) -> &[Entry] {
        let bounds = self.bounds(key);
        match self.index_of(slot) {
            Some(index) if bounds.contains(&index) => &self.entries[bounds.start..index],
            _ => &self.entries[bounds],
        }
    }

    pub fn at_node(&self, node: u32) -> &[Entry] {
        let group = node as usize;
        match (self.node_first.get(group), self.node_first.get(group + 1)) {
            (Some(&a), Some(&b)) => &self.entries[a as usize..b as usize],
            _ => &[],
        }
    }

    fn bounds(&self, key: u64) -> std::ops::Range<usize> {
        let first = if key & MOVEMENT_BIT == 0 {
            &self.link_first
        } else {
            &self.node_first
        };
        let group = group_of(key);
        let (Some(&a), Some(&b)) = (first.get(group), first.get(group + 1)) else {
            return 0..0;
        };
        let (a, b) = (a as usize, b as usize);
        let run = &self.entries[a..b];
        a + run.partition_point(|entry| entry.key < key)
            ..a + run.partition_point(|entry| entry.key <= key)
    }

    fn last_sorted(&self, key: u64) -> Option<Entry> {
        self.bounds(key)
            .rev()
            .find(|&i| self.index_of(self.entries[i].slot) == Some(i))
            .map(|i| self.entries[i])
    }
}

fn group_of(key: u64) -> usize {
    if key & MOVEMENT_BIT == 0 {
        return (key >> 8) as usize;
    }
    ((key & !MOVEMENT_BIT) >> 16) as usize
}

fn fill_offsets(first: &mut Vec<u32>, entries: &[Entry], base: usize) {
    let groups = entries.last().map_or(0, |entry| group_of(entry.key) + 1);
    first.clear();
    first.resize(groups + 1, 0);
    for entry in entries {
        first[group_of(entry.key) + 1] += 1;
    }
    let mut total = base as u32;
    for offset in first.iter_mut() {
        total += *offset;
        *offset = total;
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
