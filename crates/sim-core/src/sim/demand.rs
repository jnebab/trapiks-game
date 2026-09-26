use crate::config::{SimConfig, SimMode};
use crate::consts::{CITY_TRIP_BAND, DT, REGION_TRIP_BAND, SPAWN_ROUTE_BUDGET};
use crate::demand::{DemandTables, LinkQueues, RoutedTrip, Trip, TripClock, TripQueue, draw_trip};
use crate::network::Network;
use crate::rng::Pcg32;
use crate::routing::{LinkCosts, RouteGraph};
use crate::vehicle::SpawnError;

use super::Sim;

pub(super) struct Demand {
    pub tables: DemandTables,
    pub clock: TripClock,
    pub trips: TripQueue,
    pub queues: LinkQueues,
    pub band: (f64, f64),
}

impl Demand {
    pub fn new(
        network: &Network,
        graph: &RouteGraph,
        costs: &LinkCosts,
        config: &SimConfig,
        rng: &mut Pcg32,
    ) -> Demand {
        let band = match config.mode {
            SimMode::City => CITY_TRIP_BAND,
            SimMode::Region { .. } => REGION_TRIP_BAND,
        };
        Demand {
            tables: DemandTables::build(network, graph, costs),
            clock: TripClock::new(config.vehicles_per_hour, rng),
            trips: TripQueue::default(),
            queues: LinkQueues::default(),
            band,
        }
    }
}

impl Sim {
    pub fn set_demand(&mut self, vehicles_per_hour: f64) {
        let time_s = self.tick as f64 * DT;
        self.demand
            .clock
            .set_rate(vehicles_per_hour, time_s, &mut self.rng);
    }

    #[cfg(feature = "fixtures")]
    pub fn set_trip_band_for_test(&mut self, band: (f64, f64)) {
        self.demand.band = band;
    }

    pub fn demand_tables(&self) -> &DemandTables {
        &self.demand.tables
    }

    pub fn queued_trips(&self) -> usize {
        self.demand.trips.trips.len() + self.demand.queues.len()
    }

    pub(super) fn rebuild_demand_tables(&mut self) {
        self.demand.tables = DemandTables::build(&self.network, &self.graph, &self.costs);
    }

    pub(super) fn create_trips(&mut self) {
        let time_s = self.tick as f64 * DT;
        while self.demand.clock.is_due(time_s) {
            self.create_trip();
            self.demand.clock.advance(&mut self.rng);
        }
    }

    fn create_trip(&mut self) {
        self.stats.created += 1;
        let demand = &self.demand;
        match draw_trip(
            &demand.tables,
            &self.network,
            demand.band,
            self.tick,
            &mut self.rng,
        ) {
            Some(trip) => self.demand.trips.trips.push_back(trip),
            None => self.stats.unserved += 1,
        }
    }

    pub(super) fn route_trips(&mut self) {
        self.stats.unserved += self.demand.trips.drop_expired(self.tick);
        for _ in 0..SPAWN_ROUTE_BUDGET {
            let Some(trip) = self.demand.trips.trips.pop_front() else {
                return;
            };
            self.route_trip(trip);
        }
    }

    fn route_trip(&mut self, trip: Trip) {
        if !self.route_to_buffer(trip.from, trip.to) {
            self.stats.unserved += 1;
            return;
        }
        let route = std::mem::take(&mut self.route_buf);
        self.demand.queues.push(RoutedTrip { trip, route });
    }

    pub(super) fn spawn_queued(&mut self) {
        let mut queues = std::mem::take(&mut self.demand.queues);
        let outcome = queues.step(self.tick, |routed| self.spawn_routed(routed));
        self.demand.queues = queues;
        self.stats.unserved += outcome.unserved;
    }

    fn spawn_routed(&mut self, routed: &RoutedTrip) -> Result<u32, SpawnError> {
        let id = self.spawn_kind(&routed.route, routed.trip.kind)?;
        let waited = self.tick.saturating_sub(routed.trip.created_tick);
        self.stats.accrued_delay += waited as f64 * DT;
        Ok(id)
    }
}
