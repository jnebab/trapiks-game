use crate::vehicle::ahead::Ahead;

use super::Sim;

impl Sim {
    pub(super) fn refresh_ahead(&mut self) {
        for slot in 0..self.vehicles.slot_count() as u32 {
            if self.vehicles.alive[slot as usize] {
                self.refresh_ahead_of(slot);
            }
        }
    }

    fn refresh_ahead_of(&mut self, slot: u32) {
        let index = slot as usize;
        let place = self.vehicles.place[index];
        let cursor = self.vehicles.cursor(slot);
        if self.vehicles.ahead[index].is_fresh(&self.network, place, cursor) {
            return;
        }
        let route = self.vehicles.route(slot);
        self.vehicles.ahead[index] = Ahead::compute(&self.network, route, place, cursor);
    }
}
