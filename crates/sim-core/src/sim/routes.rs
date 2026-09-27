use crate::consts::{EMA_REFRESH_TICKS, ROUTE_HEURISTIC_WEIGHT};
use crate::network::{Changes, LinkId};
use crate::routing::{RouteContext, RouteGraph, RouteStats};
use crate::vehicle::{Place, SpawnError};

use super::Sim;

impl Sim {
    pub fn route(&mut self, from: LinkId, to: LinkId) -> Option<Vec<LinkId>> {
        self.route_with_weight(from, to, ROUTE_HEURISTIC_WEIGHT)
    }

    pub fn route_with_weight(
        &mut self,
        from: LinkId,
        to: LinkId,
        heuristic_weight: f64,
    ) -> Option<Vec<LinkId>> {
        self.ensure_graph();
        self.weighted_route_to_buffer(from, to, heuristic_weight)
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

    pub fn landmark_count(&self) -> usize {
        self.landmarks.count()
    }

    pub fn stranded(&self) -> u64 {
        self.stranded
    }

    pub(super) fn ensure_graph(&mut self) {
        if self.network.version() == self.graph.built_version() {
            return;
        }
        match self.network.changes_since(self.graph.built_version()) {
            Some(changes) => self.patch_graph(&changes),
            None => self.rebuild_graph(),
        }
        self.extend_reference_speeds();
    }

    fn rebuild_graph(&mut self) {
        self.graph = RouteGraph::build(&self.network);
        self.costs.rebuild(&self.network);
        if self.network.region().is_none() {
            self.rebuild_demand_tables();
        }
    }

    fn patch_graph(&mut self, changes: &Changes) {
        self.graph.patch(&self.network, &changes.nodes);
        self.costs.refresh_links(&self.network, &changes.links);
        if self.network.region().is_none() {
            self.demand
                .tables
                .update(&self.network, &self.graph, &self.costs, &changes.links);
        }
    }

    pub(super) fn route_to_buffer(&mut self, from: LinkId, to: LinkId) -> bool {
        self.weighted_route_to_buffer(from, to, ROUTE_HEURISTIC_WEIGHT)
    }

    fn weighted_route_to_buffer(&mut self, from: LinkId, to: LinkId, weight: f64) -> bool {
        let ctx = RouteContext {
            graph: &self.graph,
            costs: &self.costs,
            landmarks: Some(&self.landmarks),
            network: &self.network,
            heuristic_weight: weight,
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
