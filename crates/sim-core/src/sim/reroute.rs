use crate::network::LinkId;
use crate::vehicle::Place;

use super::Sim;

impl Sim {
    pub(super) fn prefetch_and_reroute(&mut self) {
        let every = self.checked_version != self.network.version();
        for slot in 0..self.vehicles.slot_count() as u32 {
            if !self.vehicles.alive[slot as usize] {
                continue;
            }
            if every || self.vehicles.needs_check(slot) {
                self.check_route(slot);
            }
        }
        self.checked_version = self.network.version();
    }

    fn check_route(&mut self, slot: u32) {
        self.ensure_route_junctions(slot);
        if self.route_broken(slot) {
            return self.reroute(slot);
        }
        self.vehicles.mark_checked(slot);
    }

    fn route_broken(&self, slot: u32) -> bool {
        let (first, pairs) = match self.vehicles.place[slot as usize] {
            Place::Link { .. } => (0, 2),
            Place::Movement { .. } => (1, 1),
        };
        let route = self.vehicles.route(slot);
        let start = self.vehicles.cursor(slot) + first;
        route
            .get(start..)
            .unwrap_or_default()
            .windows(2)
            .take(pairs)
            .any(|pair| !self.has_movement(pair[0], pair[1]))
    }

    fn has_movement(&self, from: LinkId, to: LinkId) -> bool {
        self.network
            .junction(self.network.link_to(from))
            .is_some_and(|junction| junction.movement_index(from, to).is_some())
    }

    fn reroute(&mut self, slot: u32) {
        let Some((prefix, from, target)) = self.reroute_ends(slot) else {
            return self.strand(slot);
        };
        if !self.route_to_buffer(from, target) {
            return self.strand(slot);
        }
        let len = self.route_buf.len() + usize::from(prefix.is_some());
        if len > usize::from(u16::MAX) {
            return self.strand(slot);
        }
        self.vehicles.replace_route(slot, prefix, &self.route_buf);
        self.ensure_route_junctions(slot);
        if !self.first_transition_exists() {
            return self.strand(slot);
        }
        self.refresh_free_flow(slot);
    }

    fn reroute_ends(&self, slot: u32) -> Option<(Option<LinkId>, LinkId, LinkId)> {
        let route = self.vehicles.route(slot);
        let cursor = self.vehicles.cursor(slot);
        let target = *route.last()?;
        match self.vehicles.place[slot as usize] {
            Place::Link { .. } => Some((None, *route.get(cursor)?, target)),
            Place::Movement { .. } => {
                Some((Some(*route.get(cursor)?), *route.get(cursor + 1)?, target))
            }
        }
    }

    fn first_transition_exists(&self) -> bool {
        match *self.route_buf.as_slice() {
            [from, to, ..] => self.has_movement(from, to),
            _ => true,
        }
    }

    pub(super) fn strand(&mut self, slot: u32) {
        self.vehicles.release(slot);
        self.occupancy.forget(slot);
        self.stranded += 1;
        self.stats.stranded += 1;
    }
}
