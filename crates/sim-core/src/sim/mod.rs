mod advance;
mod conflict;
mod demand;
mod free_flow;
mod hash;
mod lookahead;
mod reroute;
mod routes;
mod rules;
mod snapshot;
mod spawn;
mod stats;

use crate::config::{SimConfig, SimMode};
use crate::consts::{DT, MAX_VEHICLES};
use crate::geom::Vec2;
use crate::map::MapData;
use crate::network::{LinkId, Network, Region};
use crate::rng::Pcg32;
use crate::routing::{Landmarks, LinkCosts, RouteGraph, Router};
use crate::stats::StatsWindow;
use crate::vehicle::approach::PriorityKey;
use crate::vehicle::idm::acceleration;
use crate::vehicle::{Occupancy, Place, VehicleStore, leader};

pub use snapshot::Snapshot;

use demand::Demand;

const RNG_STREAM: u64 = 0x5452_4150_494b_5301;

pub struct Sim {
    network: Network,
    vehicles: VehicleStore,
    occupancy: Occupancy,
    accel: Vec<f64>,
    candidates: Vec<(PriorityKey, u32)>,
    rng: Pcg32,
    tick: u64,
    graph: RouteGraph,
    costs: LinkCosts,
    landmarks: Landmarks,
    router: Router,
    route_buf: Vec<LinkId>,
    stranded: u64,
    demand: Demand,
    stats: StatsWindow,
    v_ref: Vec<f64>,
    checked_version: u64,
}

impl Sim {
    pub fn new(map: &MapData, seed: u64) -> Sim {
        Sim::with_capacity(map, seed, MAX_VEHICLES)
    }

    pub fn with_capacity(map: &MapData, seed: u64, capacity: usize) -> Sim {
        let config = SimConfig {
            seed,
            mode: SimMode::City,
            vehicles_per_hour: 0.0,
        };
        Sim::build(map, &config, capacity)
    }

    pub fn from_config(map: &MapData, config: &SimConfig) -> Sim {
        Sim::build(map, config, MAX_VEHICLES)
    }

    fn build(map: &MapData, config: &SimConfig, capacity: usize) -> Sim {
        let mut network = Network::from_map(map);
        if let SimMode::Region {
            center_x,
            center_y,
            radius,
        } = config.mode
        {
            let center = Vec2::new(center_x, center_y);
            network.set_region(Some(Region { center, radius }));
        }
        let graph = RouteGraph::build(&network);
        let costs = LinkCosts::build(&network);
        let landmarks = Landmarks::build(&network, &graph, &costs);
        let mut rng = Pcg32::new(config.seed, RNG_STREAM);
        let demand = Demand::new(&network, &graph, &costs, config, &mut rng);
        Sim {
            v_ref: network.roads.speed.clone(),
            network,
            vehicles: VehicleStore::with_capacity(capacity),
            occupancy: Occupancy::default(),
            accel: Vec::with_capacity(capacity),
            candidates: Vec::new(),
            rng,
            tick: 0,
            graph,
            costs,
            landmarks,
            router: Router::new(),
            route_buf: Vec::new(),
            stranded: 0,
            demand,
            stats: StatsWindow::default(),
            checked_version: u64::MAX,
        }
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn vehicle_count(&self) -> u32 {
        self.vehicles.live()
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    #[cfg(feature = "fixtures")]
    pub fn network_mut(&mut self) -> &mut Network {
        &mut self.network
    }

    pub fn vehicles(&self) -> &VehicleStore {
        &self.vehicles
    }

    pub fn step(&mut self) {
        self.ensure_graph();
        self.prefetch_and_reroute();
        self.create_trips();
        self.route_trips();
        self.occupancy.rebuild(&self.vehicles);
        self.spawn_queued();
        self.update_zones();
        self.decide();
        self.refresh_ahead();
        self.compute_accelerations();
        self.integrate();
        self.advance_all();
        self.accrue_delay();
        self.sample_speeds();
        self.vehicles.compact_routes();
        self.tick += 1;
    }

    fn ensure_route_junctions(&mut self, slot: u32) {
        for offset in 0..3 {
            let Some(link) = self.vehicles.route_link(slot, offset) else {
                return;
            };
            let node = self.network.link_to(link);
            if self.network.junction(node).is_none() {
                self.network.ensure_junction(node);
            }
        }
    }

    fn compute_accelerations(&mut self) {
        self.accel.clear();
        self.accel.resize(self.vehicles.slot_count(), 0.0);
        for slot in self.vehicles.live_slots() {
            self.accel[slot as usize] = self.acceleration_of(slot);
        }
    }

    fn acceleration_of(&self, slot: u32) -> f64 {
        let v = self.vehicles.v[slot as usize];
        let v0 = self.desired_speed(slot);
        match self.obstacle(slot) {
            Some((gap, leader_v)) => acceleration(v, v0, gap, v - leader_v),
            None => acceleration(v, v0, f64::INFINITY, 0.0),
        }
    }

    fn obstacle(&self, slot: u32) -> Option<(f64, f64)> {
        let vehicle = leader(&self.network, &self.vehicles, &self.occupancy, slot);
        let fixed = [self.stop_line_gap(slot), self.conflict_gap(slot)]
            .into_iter()
            .flatten()
            .map(|gap| (gap, 0.0));
        vehicle
            .into_iter()
            .chain(fixed)
            .reduce(|best, next| if next.0 < best.0 { next } else { best })
    }

    fn desired_speed(&self, slot: u32) -> f64 {
        let link = match self.vehicles.place[slot as usize] {
            Place::Link { link, .. } => Some(link),
            Place::Movement { .. } => self.vehicles.route_link(slot, 1),
        };
        link.map_or(0.0, |link| self.network.link_speed(link))
    }

    fn integrate(&mut self) {
        for slot in 0..self.vehicles.slot_count() {
            if self.vehicles.alive[slot] {
                self.integrate_slot(slot);
            }
        }
    }

    fn integrate_slot(&mut self, slot: usize) {
        let a = self.accel[slot];
        let v = self.vehicles.v[slot];
        let next_v = v + a * DT;
        if next_v < 0.0 {
            self.vehicles.s[slot] += -v * v / (2.0 * a);
            self.vehicles.v[slot] = 0.0;
            return;
        }
        self.vehicles.s[slot] += v * DT + 0.5 * a * DT * DT;
        self.vehicles.v[slot] = next_v;
    }
}
