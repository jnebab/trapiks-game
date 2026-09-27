use crate::consts::FLAGGED_REROUTE_BUDGET;
use crate::edit::vehicles::{RouteDamage, flag_routes};

use super::Sim;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RerouteCounts {
    pub immediate: u64,
    pub flagged: u64,
    pub marked: u64,
}

impl Sim {
    pub fn flagged_slots(&self) -> impl Iterator<Item = u32> + '_ {
        self.edits.flagged.iter().copied()
    }

    pub fn reroute_counts(&self) -> RerouteCounts {
        self.edits.reroutes
    }

    pub(super) fn flag_damaged(&mut self, damage: &RouteDamage) {
        if damage.is_empty() {
            return;
        }
        let marked = flag_routes(&self.vehicles, damage, &mut self.edits.flagged);
        self.edits.reroutes.marked += marked;
    }

    pub(super) fn reroute_flagged(&mut self) {
        for _ in 0..FLAGGED_REROUTE_BUDGET {
            let Some(slot) = self.edits.flagged.pop_first() else {
                return;
            };
            self.edits.reroutes.flagged += 1;
            self.reroute(slot);
        }
    }

    pub(super) fn release_slot(&mut self, slot: u32) {
        self.vehicles.release(slot);
        self.occupancy.forget(slot);
        self.edits.flagged.remove(&slot);
    }
}
