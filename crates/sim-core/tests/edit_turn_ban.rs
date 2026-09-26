mod support;

use std::collections::BTreeMap;

use support::{apply_now, grid_10};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, Outcome};
use trapiks_sim_core::network::road_of;
use trapiks_sim_core::sim::Sim;

fn busy_sim() -> Sim {
    let config = SimConfig {
        seed: 11,
        mode: SimMode::City,
        vehicles_per_hour: 8_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(&grid_10(), &config);
    sim.set_trip_band_for_test((600.0, 3_000.0));
    for _ in 0..600 {
        sim.step();
    }
    sim
}

fn distant_pairs(sim: &Sim) -> BTreeMap<(u32, u32), usize> {
    let vehicles = sim.vehicles();
    let mut counts = BTreeMap::new();
    for slot in vehicles.live_slots() {
        let route = vehicles.route(slot);
        let rest = route.get(vehicles.cursor(slot) + 2..).unwrap_or_default();
        for pair in rest.windows(2) {
            *counts.entry((pair[0], pair[1])).or_default() += 1;
        }
    }
    counts
}

fn busiest_pair(sim: &Sim) -> Option<(u32, u32)> {
    distant_pairs(sim)
        .into_iter()
        .max_by_key(|&(pair, count)| (count, std::cmp::Reverse(pair)))
        .map(|(pair, _)| pair)
}

fn ban(sim: &Sim, (from, to): (u32, u32)) -> EditCommand {
    EditCommand::SetTurnAllowed {
        node: sim.network().link_to(from),
        from_road: road_of(from),
        to_road: road_of(to),
        allowed: false,
    }
}

fn contains_pair(sim: &Sim, slot: u32, pair: (u32, u32)) -> bool {
    let route = sim.vehicles().route(slot);
    route
        .get(sim.vehicles().cursor(slot)..)
        .unwrap_or_default()
        .windows(2)
        .any(|w| (w[0], w[1]) == pair)
}

#[test]
fn turn_ban_edit() {
    let mut sim = busy_sim();
    let pair = busiest_pair(&sim).expect("routes");
    let expected = distant_pairs(&sim)[&pair];
    assert!(expected >= 5, "{expected}");
    let before = sim.reroute_counts();
    let command = ban(&sim, pair);
    assert!(matches!(apply_now(&mut sim, command), Outcome::Ok(_)));
    let marked = sim.reroute_counts().marked - before.marked;
    assert!(marked >= expected as u64, "{marked} < {expected}");
    let deadline = marked.div_ceil(4);
    let mut steps = 1;
    loop {
        let rerouted = sim.reroute_counts().flagged - before.flagged;
        assert!(rerouted <= 4 * steps, "{rerouted} after {steps} steps");
        if sim.flagged_slots().count() == 0 {
            break;
        }
        assert!(steps < deadline, "flagged left after {steps} steps");
        sim.step();
        steps += 1;
    }
    for slot in sim.vehicles().live_slots() {
        assert!(
            !contains_pair(&sim, slot, pair),
            "slot {slot} kept the banned turn"
        );
    }
}
