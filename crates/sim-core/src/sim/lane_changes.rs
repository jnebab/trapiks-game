use crate::vehicle::Place;
use crate::vehicle::ahead::Ahead;
use crate::vehicle::lane_change::{choose, is_safe};

use super::Sim;

impl Sim {
    pub fn lane_change_count(&self) -> u64 {
        self.lane_change_count
    }

    #[cfg(feature = "fixtures")]
    pub fn set_lane_changes_for_test(&mut self, enabled: bool) {
        self.lane_changes_enabled = enabled;
    }

    #[cfg(feature = "fixtures")]
    pub fn place_vehicle_for_test(&mut self, slot: u32, lane: u8) {
        let index = slot as usize;
        let Place::Link { link, .. } = self.vehicles.place[index] else {
            return;
        };
        self.vehicles.set_place(slot, Place::Link { link, lane });
        self.occupancy.rebuild(&self.vehicles);
    }

    pub(super) fn change_lanes(&mut self) {
        if !self.lane_changes_enabled {
            return;
        }
        let mut picks = std::mem::take(&mut self.lane_picks);
        picks.clear();
        let ctx = self.rule_context();
        for slot in self.vehicles.live_slots() {
            if let Some(target) = choose(&ctx, slot) {
                picks.push((slot, target));
            }
        }
        for &(slot, target) in &picks {
            if is_safe(&self.rule_context(), slot, target) {
                self.apply_lane_change(slot, target);
            }
        }
        self.lane_picks = picks;
    }

    fn apply_lane_change(&mut self, slot: u32, target: u8) {
        let index = slot as usize;
        let Place::Link { link, lane } = self.vehicles.place[index] else {
            return;
        };
        self.vehicles.lane_from[index] = lane;
        self.vehicles.place[index] = Place::Link { link, lane: target };
        self.vehicles.lane_change_tick[index] = self.tick;
        self.occupancy.move_to_lane(slot, link, target);
        let route = self.vehicles.route(slot);
        let cursor = self.vehicles.cursor(slot);
        let place = self.vehicles.place[index];
        self.vehicles.ahead[index] = Ahead::compute(&self.network, route, place, cursor);
        self.lane_change_count += 1;
    }
}
