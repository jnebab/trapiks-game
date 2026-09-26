use crate::consts::EMA_REFRESH_TICKS;
use crate::network::LinkId;
use crate::routing::{RouteContext, RouteGraph, RouteStats};
use crate::vehicle::{Place, SpawnError};

use super::Sim;

impl Sim {
    pub fn route(&mut self, from: LinkId, to: LinkId) -> Option<Vec<LinkId>> {
        self.ensure_graph();
        self.route_to_buffer(from, to)
            .then(|| self.route_buf.clone())
    }

    pub fn spawn_trip(&mut self, from: LinkId, to: LinkId) -> Result<u32, SpawnError> {
        self.ensure_graph();
        if !self.route_to_buffer(from, to) {
            return Err(SpawnError::NoRoute);
        }
        let route = std::mem::take(&mut self.route_buf);
        let result = self.spawn(&route);
        self.route_buf = route;
        result
    }

    pub fn route_stats(&self) -> RouteStats {
        self.router.stats()
    }

    pub fn stranded(&self) -> u64 {
        self.stranded
    }

    pub(super) fn ensure_graph(&mut self) {
        if self.network.version() == self.graph.built_version() {
            return;
        }
        self.graph = RouteGraph::build(&self.network);
        self.costs.grow(&self.network);
    }

    pub(super) fn route_to_buffer(&mut self, from: LinkId, to: LinkId) -> bool {
        let ctx = RouteContext {
            graph: &self.graph,
            costs: &self.costs,
            landmarks: Some(&self.landmarks),
            network: &self.network,
        };
        self.router.route_into(ctx, from, to, &mut self.route_buf)
    }

    pub(super) fn sample_speeds(&mut self) {
        for slot in 0..self.vehicles.slot_count() {
            if !self.vehicles.alive[slot] {
                continue;
            }
            if let Place::Link { link, .. } = self.vehicles.place[slot] {
                self.costs.accumulate(link, self.vehicles.v[slot]);
            }
        }
        if self.tick.is_multiple_of(EMA_REFRESH_TICKS) && self.tick > 0 {
            self.costs.refresh();
        }
    }
}
