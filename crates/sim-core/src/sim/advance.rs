use crate::vehicle::leader::next_place;
use crate::vehicle::{Place, entry_of, place_end};

use super::Sim;

impl Sim {
    pub(super) fn advance_all(&mut self) {
        for slot in 0..self.vehicles.slot_count() as u32 {
            if self.vehicles.alive[slot as usize] {
                self.advance_one(slot);
            }
        }
    }

    fn advance_one(&mut self, slot: u32) {
        let before = self.vehicles.place[slot as usize];
        if !self.advance(slot) {
            self.vehicles.release(slot);
            self.occupancy.forget(slot);
            return;
        }
        if self.vehicles.place[slot as usize] != before {
            self.occupancy.push_pending(entry_of(&self.vehicles, slot));
        }
    }

    fn advance(&mut self, slot: u32) -> bool {
        let index = slot as usize;
        loop {
            let place = self.vehicles.place[index];
            let Some(end) = place_end(&self.network, place) else {
                return false;
            };
            let s = self.vehicles.s[index];
            if s <= end {
                return true;
            }
            self.ensure_route_junctions(slot);
            let route = self.vehicles.route(slot);
            let cursor = self.vehicles.cursor(slot);
            let Some((next, next_cursor)) = next_place(&self.network, route, place, cursor) else {
                return false;
            };
            self.vehicles.place[index] = next;
            self.vehicles.route_cursor[index] = next_cursor as u16;
            self.vehicles.s[index] = self.place_start(next) + s - end;
        }
    }

    fn place_start(&self, place: Place) -> f64 {
        match place {
            Place::Link { link, .. } => self.network.link_span(link).0,
            Place::Movement { .. } => 0.0,
        }
    }
}
