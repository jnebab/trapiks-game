use crate::vehicle::leader::next_place;
use crate::vehicle::{Place, entry_of, place_end};

use super::Sim;

enum Outcome {
    Alive,
    Arrived,
    Stranded,
}

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
        match self.advance(slot) {
            Outcome::Alive => {}
            Outcome::Arrived => {
                self.vehicles.release(slot);
                self.occupancy.forget(slot);
                return;
            }
            Outcome::Stranded => return self.strand(slot),
        }
        if self.vehicles.place[slot as usize] != before {
            self.occupancy.push_pending(entry_of(&self.vehicles, slot));
        }
    }

    fn advance(&mut self, slot: u32) -> Outcome {
        let index = slot as usize;
        loop {
            let place = self.vehicles.place[index];
            let Some(end) = place_end(&self.network, place) else {
                return Outcome::Stranded;
            };
            let s = self.vehicles.s[index];
            if s <= end {
                return Outcome::Alive;
            }
            if self.held_at_line(slot, place) {
                self.vehicles.s[index] = end;
                self.vehicles.v[index] = 0.0;
                return Outcome::Alive;
            }
            if self.at_destination(slot, place) {
                return Outcome::Arrived;
            }
            self.ensure_route_junctions(slot);
            let route = self.vehicles.route(slot);
            let cursor = self.vehicles.cursor(slot);
            let Some((next, next_cursor)) = next_place(&self.network, route, place, cursor) else {
                return Outcome::Stranded;
            };
            self.vehicles.place[index] = next;
            self.vehicles.route_cursor[index] = next_cursor as u16;
            self.vehicles.s[index] = self.place_start(next) + s - end;
            self.vehicles.reset_junction_state(slot);
        }
    }

    fn at_destination(&self, slot: u32, place: Place) -> bool {
        matches!(place, Place::Link { .. }) && self.vehicles.route_link(slot, 1).is_none()
    }

    fn held_at_line(&self, slot: u32, place: Place) -> bool {
        matches!(place, Place::Link { .. })
            && !self.vehicles.committed[slot as usize]
            && self.vehicles.route_link(slot, 1).is_some()
    }

    fn place_start(&self, place: Place) -> f64 {
        match place {
            Place::Link { link, .. } => self.network.link_span(link).0,
            Place::Movement { .. } => 0.0,
        }
    }
}
