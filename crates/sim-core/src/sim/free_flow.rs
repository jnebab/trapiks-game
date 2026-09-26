use crate::consts::{DT, MIN_LINK_SPEED};
use crate::network::LinkId;
use crate::vehicle::{Place, place_end};

use super::Sim;

impl Sim {
    pub(super) fn route_free_time(&self, links: &[LinkId]) -> f64 {
        links.iter().map(|&link| self.costs.free_time(link)).sum()
    }

    pub(super) fn refresh_free_flow(&mut self, slot: u32) {
        let index = slot as usize;
        let elapsed = self.tick.saturating_sub(self.vehicles.spawn_tick[index]) as f64 * DT;
        let cursor = self.vehicles.cursor(slot);
        let rest = self
            .vehicles
            .route(slot)
            .get(cursor + 1..)
            .unwrap_or_default();
        let remaining = self.place_free_time(slot) + self.route_free_time(rest);
        self.vehicles.free_flow[index] = elapsed + remaining;
    }

    fn place_free_time(&self, slot: u32) -> f64 {
        let index = slot as usize;
        let place = self.vehicles.place[index];
        let s = self.vehicles.s[index];
        let (end, link) = match place {
            Place::Link { link, .. } => (self.network.link_span(link).1, Some(link)),
            Place::Movement { .. } => (
                place_end(&self.network, place).unwrap_or(s),
                self.vehicles.route_link(slot, 1),
            ),
        };
        let speed = link.map_or(MIN_LINK_SPEED, |link| self.free_speed(link));
        (end - s).max(0.0) / speed
    }

    pub(super) fn free_speed(&self, link: LinkId) -> f64 {
        self.costs
            .free_speed
            .get(link as usize)
            .map_or(MIN_LINK_SPEED, |&speed| speed.max(MIN_LINK_SPEED))
    }
}
