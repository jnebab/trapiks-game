mod advance;
mod hash;
mod snapshot;
mod spawn;

use crate::consts::{DT, MAX_VEHICLES};
use crate::map::MapData;
use crate::network::{Network, road_of};
use crate::rng::Pcg32;
use crate::vehicle::idm::acceleration;
use crate::vehicle::{Occupancy, Place, VehicleStore, leader};

pub use snapshot::Snapshot;

const RNG_STREAM: u64 = 0x5452_4150_494b_5301;

pub struct Sim {
    network: Network,
    vehicles: VehicleStore,
    occupancy: Occupancy,
    accel: Vec<f64>,
    rng: Pcg32,
    tick: u64,
}

impl Sim {
    pub fn new(map: &MapData, seed: u64) -> Sim {
        Sim::with_capacity(map, seed, MAX_VEHICLES)
    }

    pub fn with_capacity(map: &MapData, seed: u64, capacity: usize) -> Sim {
        Sim {
            network: Network::from_map(map),
            vehicles: VehicleStore::with_capacity(capacity),
            occupancy: Occupancy::default(),
            accel: Vec::with_capacity(capacity),
            rng: Pcg32::new(seed, RNG_STREAM),
            tick: 0,
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

    pub fn vehicles(&self) -> &VehicleStore {
        &self.vehicles
    }

    pub fn step(&mut self) {
        self.ensure_all_route_junctions();
        self.occupancy.rebuild(&self.vehicles);
        self.compute_accelerations();
        self.integrate();
        self.advance_all();
        self.vehicles.compact_routes();
        self.tick += 1;
    }

    fn ensure_all_route_junctions(&mut self) {
        for slot in 0..self.vehicles.slot_count() as u32 {
            if self.vehicles.alive[slot as usize] {
                self.ensure_route_junctions(slot);
            }
        }
    }

    fn ensure_route_junctions(&mut self, slot: u32) {
        for offset in 0..2 {
            if let Some(link) = self.vehicles.route_link(slot, offset) {
                let node = self.network.link_to(link);
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
        match leader(&self.network, &self.vehicles, &self.occupancy, slot) {
            Some((gap, leader_v)) => acceleration(v, v0, gap, v - leader_v),
            None => acceleration(v, v0, f64::INFINITY, 0.0),
        }
    }

    fn desired_speed(&self, slot: u32) -> f64 {
        let link = match self.vehicles.place[slot as usize] {
            Place::Link { link, .. } => Some(link),
            Place::Movement { .. } => self.vehicles.route_link(slot, 1),
        };
        link.map_or(0.0, |link| self.network.roads.speed[road_of(link) as usize])
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
