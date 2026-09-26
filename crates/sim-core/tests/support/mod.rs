#![allow(dead_code)]

use std::collections::BTreeMap;

use trapiks_sim_core::map::MapData;
use trapiks_sim_core::network::Network;
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, place_key};

const BODY_MARGIN: f64 = 1.6;

pub fn run_until(sim: &mut Sim, max_ticks: u64, mut predicate: impl FnMut(&Sim) -> bool) -> bool {
    for _ in 0..max_ticks {
        if predicate(sim) {
            return true;
        }
        sim.step();
    }
    predicate(sim)
}

pub fn slot_of(sim: &Sim, id: u32) -> Option<u32> {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .find(|&slot| vehicles.id[slot as usize] == id)
}

pub fn has_committed(sim: &Sim, id: u32) -> bool {
    let Some(slot) = slot_of(sim, id) else {
        return true;
    };
    let vehicles = sim.vehicles();
    let index = slot as usize;
    vehicles.committed[index]
        || vehicles.route_cursor[index] > 0
        || !matches!(vehicles.place[index], Place::Link { .. })
}

#[derive(Default)]
pub struct Commits {
    ticks: BTreeMap<u32, Option<u64>>,
}

impl Commits {
    pub fn track(&mut self, id: u32) {
        self.ticks.insert(id, None);
    }

    pub fn step(&mut self, sim: &mut Sim) {
        let tick = sim.tick();
        sim.step();
        for (&id, slot) in &mut self.ticks {
            if slot.is_none() && has_committed(sim, id) {
                *slot = Some(tick);
            }
        }
    }

    pub fn tick_of(&self, id: u32) -> Option<u64> {
        self.ticks.get(&id).copied().flatten()
    }
}

pub fn check_invariants(sim: &Sim) {
    let vehicles = sim.vehicles();
    let mut groups: BTreeMap<u64, Vec<(f64, f64)>> = BTreeMap::new();
    for slot in vehicles.live_slots() {
        let index = slot as usize;
        let (s, v) = (vehicles.s[index], vehicles.v[index]);
        assert!(!s.is_nan() && !v.is_nan(), "NaN on slot {slot}");
        assert!(v >= 0.0, "negative speed on slot {slot}");
        groups
            .entry(place_key(vehicles.place[index]))
            .or_default()
            .push((s, vehicles.length(slot)));
    }
    for (key, mut list) in groups {
        list.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in list.windows(2) {
            let spacing = pair[1].0 - pair[0].0;
            assert!(
                spacing >= pair[1].1,
                "spacing {spacing} on key {key:#x} at tick {}",
                sim.tick()
            );
        }
    }
    check_crossings(sim);
}

fn movement_vehicles(sim: &Sim) -> Vec<(u32, u16, f64, f64)> {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter_map(|slot| match vehicles.place[slot as usize] {
            Place::Movement { node, movement, .. } => Some((
                node,
                movement,
                vehicles.s[slot as usize],
                vehicles.length(slot),
            )),
            Place::Link { .. } => None,
        })
        .collect()
}

fn covers(s: f64, length: f64, point: f64) -> bool {
    point >= s - length - BODY_MARGIN && point <= s + BODY_MARGIN
}

fn check_crossings(sim: &Sim) {
    let inside = movement_vehicles(sim);
    for &(node, movement, s, length) in &inside {
        let Some(junction) = sim.network().junction(node) else {
            continue;
        };
        for conflict in &junction.conflicts[usize::from(movement)] {
            if conflict.merge || !covers(s, length, conflict.s_self) {
                continue;
            }
            let clash = inside.iter().any(|&(n2, m2, s2, l2)| {
                n2 == node && m2 == conflict.other && covers(s2, l2, conflict.s_other)
            });
            assert!(
                !clash,
                "crossing overlap at node {node} movement {movement} tick {}",
                sim.tick()
            );
        }
    }
}

pub fn random_routes(map: &MapData, rng: &mut Pcg32, len: usize) -> Vec<Vec<u32>> {
    let mut network = Network::from_map(map);
    network.build_all_junctions();
    let movements = all_movements(&network);
    let starts: Vec<(u32, u32)> = movements
        .iter()
        .copied()
        .filter(|&(from, _)| !network.signals().is_internal(from))
        .collect();
    (0..len)
        .map(|_| random_route(&movements, &starts, rng))
        .collect()
}

fn all_movements(network: &Network) -> Vec<(u32, u32)> {
    (0..network.nodes.count() as u32)
        .filter_map(|node| network.junction(node))
        .flat_map(|junction| junction.movements.iter().map(|m| (m.from_link, m.to_link)))
        .collect()
}

fn random_route(movements: &[(u32, u32)], starts: &[(u32, u32)], rng: &mut Pcg32) -> Vec<u32> {
    let (from, to) = starts[rng.below(starts.len() as u32) as usize];
    let mut route = vec![from, to];
    let next: Vec<u32> = movements
        .iter()
        .filter(|&&(a, _)| a == to)
        .map(|&(_, b)| b)
        .collect();
    if !next.is_empty() && rng.below(2) == 1 {
        route.push(next[rng.below(next.len() as u32) as usize]);
    }
    route
}

pub fn apply_now(
    sim: &mut Sim,
    command: trapiks_sim_core::edit::EditCommand,
) -> trapiks_sim_core::edit::Outcome {
    let seq = sim.enqueue(command);
    sim.step();
    let results = sim.take_results();
    let Some(result) = results.into_iter().find(|result| result.seq == seq) else {
        panic!("no result for command {seq}");
    };
    result.outcome
}

pub fn grid_10() -> MapData {
    trapiks_sim_core::fixtures::grid_city(&trapiks_sim_core::fixtures::GridCity {
        cols: 10,
        rows: 10,
        spacing: 150.0,
    })
}

pub trait Must<T> {
    fn must(self, what: &str) -> T;
}

impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{what}: {error:?}"),
        }
    }
}

impl<T> Must<T> for Option<T> {
    fn must(self, what: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{what}: missing"),
        }
    }
}
