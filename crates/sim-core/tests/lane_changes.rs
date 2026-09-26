mod support;

use support::{Must, check_invariants, has_committed, run_until, slot_of};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::consts::WAIT_TIMEOUT_TICKS;
use trapiks_sim_core::fixtures::{GridCity, corridor, four_way, grid_city};
use trapiks_sim_core::network::Network;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, VehicleKind};

const LEFT: [u32; 2] = [0, 3];
const SOUTH_THROUGH: [u32; 2] = [4, 1];
const CAR_SPAWN_TICK: u64 = 60;
const STREAM_EVERY: u64 = 120;

fn lane_of(sim: &Sim, slot: u32) -> Option<u8> {
    match sim.vehicles().place[slot as usize] {
        Place::Link { lane, .. } => Some(lane),
        Place::Movement { .. } => None,
    }
}

fn left_lanes(map: &trapiks_sim_core::map::MapData) -> (u8, u8) {
    let mut network = Network::from_map(map);
    let junction = network.ensure_junction(0);
    let index = junction.movement_index(LEFT[0], LEFT[1]).must("left");
    junction.movements[index].from_lanes
}

#[test]
fn mandatory_change() {
    let map = four_way(3, 300.0);
    let (low, _) = left_lanes(&map);
    assert!(low > 0);
    let mut sim = Sim::new(&map, 1);
    let id = sim.spawn(&LEFT).must("spawn");
    let slot = slot_of(&sim, id).must("slot");
    sim.place_vehicle_for_test(slot, 0);
    let reached = run_until(&mut sim, 600, |sim| {
        check_invariants(sim);
        lane_of(sim, slot).is_some_and(|lane| lane >= low)
    });
    assert!(reached);
    assert_eq!(sim.vehicles().arrival_tick[slot as usize], u64::MAX);
    assert!(sim.lane_change_count() >= u64::from(low));
}

fn pass_state(sim: &Sim, car: u32, bus: u32) -> Option<(u8, bool)> {
    let car_slot = slot_of(sim, car)?;
    let lane = lane_of(sim, car_slot)?;
    let Some(bus_slot) = slot_of(sim, bus) else {
        return Some((lane, true));
    };
    let v = sim.vehicles();
    let ahead = (v.route_cursor[car_slot as usize], v.s[car_slot as usize])
        > (v.route_cursor[bus_slot as usize], v.s[bus_slot as usize]);
    Some((lane, ahead))
}

#[test]
fn mobil_overtakes() {
    let mut sim = Sim::new(&corridor(), 1);
    let route = [0, 2, 4];
    let bus = sim.spawn_kind(&route, VehicleKind::Bus).must("bus");
    run_until(&mut sim, CAR_SPAWN_TICK, |_| false);
    let car = sim.spawn(&route).must("car");
    let car_slot = slot_of(&sim, car).must("car slot");
    sim.place_vehicle_for_test(car_slot, 0);
    let changed = run_until(&mut sim, 600, |sim| {
        check_invariants(sim);
        pass_state(sim, car, bus).is_some_and(|(lane, _)| lane == 1)
    });
    assert!(changed, "no change to lane 1");
    let passed = run_until(&mut sim, 2_000, |sim| {
        check_invariants(sim);
        pass_state(sim, car, bus).is_some_and(|(_, ahead)| ahead)
    });
    assert!(passed, "never passed the bus");
    let returned = run_until(&mut sim, 2_000, |sim| {
        check_invariants(sim);
        pass_state(sim, car, bus).is_some_and(|(lane, _)| lane == 0)
    });
    assert!(returned, "never returned right");
}

struct Fallback {
    sim: Sim,
    turner: u32,
    stream: Vec<u32>,
    arrival: u64,
    committed_at: Option<u64>,
}

impl Fallback {
    fn new() -> Fallback {
        let mut sim = Sim::new(&four_way(3, 300.0), 1);
        sim.set_lane_changes_for_test(false);
        let turner = sim.spawn(&LEFT).must("turner");
        let slot = slot_of(&sim, turner).must("slot");
        sim.place_vehicle_for_test(slot, 0);
        Fallback {
            sim,
            turner,
            stream: Vec::new(),
            arrival: u64::MAX,
            committed_at: None,
        }
    }

    fn step(&mut self) {
        if self.sim.tick().is_multiple_of(STREAM_EVERY)
            && let Ok(id) = self.sim.spawn(&SOUTH_THROUGH)
        {
            self.stream.push(id);
        }
        self.sim.step();
        check_invariants(&self.sim);
    }

    fn observe(&mut self) {
        if let Some(slot) = self.turner_slot() {
            self.observe_turner(slot);
        }
        if self.arrival == u64::MAX || self.sim.tick() < self.arrival + WAIT_TIMEOUT_TICKS {
            assert!(
                !self.stream_waits(),
                "stream yielded to an excluded vehicle"
            );
        }
        if self.committed_at.is_none() && has_committed(&self.sim, self.turner) {
            self.committed_at = Some(self.sim.tick());
        }
    }

    fn observe_turner(&mut self, slot: u32) {
        let vehicles = self.sim.vehicles();
        self.arrival = self.arrival.min(vehicles.arrival_tick[slot as usize]);
        if vehicles.route_cursor[slot as usize] == 0 {
            assert_eq!(lane_of(&self.sim, slot).unwrap_or(0), 0);
        }
    }

    fn stream_waits(&self) -> bool {
        let vehicles = self.sim.vehicles();
        self.stream
            .iter()
            .filter_map(|&id| slot_of(&self.sim, id))
            .any(|slot| vehicles.wait_ticks[slot as usize] > 0)
    }

    fn turner_slot(&self) -> Option<u32> {
        slot_of(&self.sim, self.turner)
    }
}

#[test]
fn wrong_lane_fallback() {
    let mut run = Fallback::new();
    for _ in 0..1_500 {
        run.step();
        run.observe();
        if run.turner_slot().is_none() {
            break;
        }
    }
    let committed_at = run.committed_at.must("turner committed");
    assert!(run.arrival != u64::MAX);
    assert!(committed_at >= run.arrival + WAIT_TIMEOUT_TICKS);
    assert!(run.turner_slot().is_none(), "turn not completed");
    assert!(!run.stream.is_empty());
}

#[test]
fn multi_lane_entry() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let first = sim.spawn(&[0, 5]).must("first");
    let second = sim.spawn(&[0, 5]).must("second");
    let (a, b) = (
        slot_of(&sim, first).must("a"),
        slot_of(&sim, second).must("b"),
    );
    assert_ne!(lane_of(&sim, a), lane_of(&sim, b));
    assert_eq!(sim.vehicles().s[a as usize], sim.vehicles().s[b as usize]);
    let mut order = Vec::new();
    let done = run_until(&mut sim, 3_000, |sim| {
        check_invariants(sim);
        for id in [first, second] {
            let entered = slot_of(sim, id).is_none_or(|slot| {
                lane_of(sim, slot).is_none() || sim.vehicles().route_cursor[slot as usize] > 0
            });
            if entered && !order.contains(&id) {
                order.push(id);
            }
        }
        order.len() == 2
    });
    assert!(done);
    let lower_first = if a < b { first } else { second };
    assert_eq!(order[0], lower_first);
}

#[test]
fn lane_change_safety() {
    let map = grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed: 3,
        mode: SimMode::City,
        vehicles_per_hour: 12_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    for _ in 0..3_000 {
        sim.step();
        check_invariants(&sim);
    }
    assert!(sim.lane_change_count() >= 50, "{}", sim.lane_change_count());
}
