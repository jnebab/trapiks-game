mod support;

use std::collections::BTreeMap;
use std::time::Instant;

use support::{check_invariants, random_routes};
use trapiks_sim_core::fixtures::{
    MapBuilder, RoadSpec, dual_carriageway_cross, four_way, one_way_pair, t_junction,
};
use trapiks_sim_core::map::MapData;
use trapiks_sim_core::map::{Control, RoadClass};
use trapiks_sim_core::network::{Direction, link_id};
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::sim::Sim;

const MAX_AGE: u64 = 3_000;
const ROUTE_POOL: usize = 64;

struct Run {
    sim: Sim,
    routes: Vec<Vec<u32>>,
    rng: Pcg32,
    spawned: BTreeMap<u32, u64>,
    step_nanos: Vec<u128>,
}

impl Run {
    fn new(map: &MapData) -> Run {
        let mut rng = Pcg32::new(2024, 7);
        let routes = random_routes(map, &mut rng, ROUTE_POOL);
        Run {
            sim: Sim::new(map, 1),
            routes,
            rng,
            spawned: BTreeMap::new(),
            step_nanos: Vec::new(),
        }
    }

    fn spawn_random(&mut self) {
        let route = &self.routes[self.rng.below(self.routes.len() as u32) as usize];
        if let Ok(id) = self.sim.spawn(route) {
            self.spawned.insert(id, self.sim.tick());
        }
    }

    fn step(&mut self) {
        let start = Instant::now();
        self.sim.step();
        self.step_nanos.push(start.elapsed().as_nanos());
        check_invariants(&self.sim);
        self.check_ages();
    }

    fn check_ages(&self) {
        let vehicles = self.sim.vehicles();
        for slot in vehicles.live_slots() {
            let age = self.sim.tick() - vehicles.spawn_tick[slot as usize];
            assert!(
                age <= MAX_AGE,
                "vehicle {} is {age} ticks old",
                vehicles.id[slot as usize]
            );
        }
    }

    fn drive(&mut self, ticks: u64, every: u64, mut on_tick: impl FnMut(&Sim)) {
        while self.sim.tick() < ticks {
            if self.sim.tick().is_multiple_of(every) {
                self.spawn_random();
            }
            self.step();
            on_tick(&self.sim);
        }
        let drain_until = ticks + MAX_AGE;
        while self.sim.vehicle_count() > 0 && self.sim.tick() < drain_until {
            self.step();
        }
        assert_eq!(self.sim.vehicle_count(), 0);
    }

    fn median_step_micros(&mut self) -> f64 {
        self.step_nanos.sort_unstable();
        let median = self
            .step_nanos
            .get(self.step_nanos.len() / 2)
            .copied()
            .unwrap_or(0);
        median as f64 / 1_000.0
    }
}

fn internal_links_unsignalled(sim: &Sim) {
    let network = sim.network();
    let links = network.roads.count() as u32 * 2;
    for link in (0..links).filter(|&link| network.signals().is_internal(link)) {
        assert_eq!(network.signal_state(link, sim.tick()), None);
    }
}

#[test]
fn cluster_flow() {
    let mut run = Run::new(&dual_carriageway_cross());
    run.drive(3_000, 20, internal_links_unsignalled);
    assert!(!run.spawned.is_empty());
    eprintln!(
        "cluster_flow median step {:.1} us",
        run.median_step_micros()
    );
}

#[test]
fn low_demand_all_fixtures() {
    let maps = [
        four_way(2, 200.0),
        t_junction(),
        one_way_pair(),
        dual_carriageway_cross(),
    ];
    for map in &maps {
        let mut run = Run::new(map);
        run.drive(6_000, 40, |_| {});
        assert!(!run.spawned.is_empty());
    }
}

fn short_exit_cluster() -> MapData {
    let mut b = MapBuilder::new();
    let west = b.node(-200.0, 0.0);
    let a = b.node(0.0, 0.0);
    let east = b.node(20.0, 0.0);
    let exit = b.node(24.0, 0.0);
    let north = b.node(0.0, -200.0);
    let south = b.node(20.0, 200.0);
    b.control(a, Control::Signal);
    b.control(east, Control::Signal);
    let spec = RoadSpec::new(RoadClass::Primary, 1, 1);
    b.road(west, a, spec.clone());
    b.road(a, east, spec.clone());
    b.road(east, exit, spec.clone());
    b.road(north, a, spec.clone());
    b.road(south, east, spec);
    b.build()
}

#[test]
fn short_cluster_exit() {
    let mut sim = Sim::new(&short_exit_cluster(), 1);
    let route = [0, 1, 2].map(|road| link_id(road, Direction::Forward));
    let id = sim.spawn(&route).expect("spawn");
    let left = support::run_until(&mut sim, 600, |sim| {
        check_invariants(sim);
        support::slot_of(sim, id).is_none()
    });
    assert!(left, "vehicle {id} did not leave the exit");
}
