mod support;

use support::{Commits, check_invariants, slot_of};
use trapiks_sim_core::consts::WAIT_TIMEOUT_TICKS;
use trapiks_sim_core::fixtures::{four_way, t_junction};
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::network::road_of;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const MINOR: [u32; 2] = [4, 1];
const MAJOR: [u32; 2] = [0, 2];
const NEAR_NODE: f64 = 20.0;
const TIMEOUT_MINOR_TICK: u64 = 30;

fn with_control(mut map: MapData, control: Control) -> MapData {
    map.nodes.control[0] = control;
    map
}

fn distance_to_node(sim: &Sim, slot: u32) -> Option<f64> {
    let vehicles = sim.vehicles();
    let index = slot as usize;
    let s = vehicles.s[index];
    match vehicles.place[index] {
        Place::Movement { .. } => Some(0.0),
        Place::Link { link: 0, .. } => Some(sim.network().link_length(0) - s),
        Place::Link { link: 2, .. } => Some(s),
        Place::Link { .. } => None,
    }
}

fn major_passed(sim: &Sim, id: u32) -> bool {
    let Some(slot) = slot_of(sim, id) else {
        return true;
    };
    let vehicles = sim.vehicles();
    match vehicles.place[slot as usize] {
        Place::Link { link, .. } => link == 2,
        Place::Movement { node, movement, .. } => {
            let Some(junction) = sim.network().junction(node) else {
                return false;
            };
            let minor = junction.movement_index(MINOR[0], MINOR[1]);
            junction.conflicts[usize::from(movement)]
                .iter()
                .filter(|c| Some(usize::from(c.other)) == minor)
                .all(|c| vehicles.s[slot as usize] - vehicles.length(slot) > c.s_self)
        }
    }
}

fn check_major_speed(sim: &Sim, majors: &[u32]) {
    let v0 = sim.network().roads.speed[road_of(0) as usize];
    for &id in majors {
        let Some(slot) = slot_of(sim, id) else {
            continue;
        };
        if distance_to_node(sim, slot).is_some_and(|d| d <= NEAR_NODE) {
            assert!(sim.vehicles().v[slot as usize] >= 0.8 * v0);
        }
    }
}

#[test]
fn t_junction_priority() {
    let mut sim = Sim::new(&t_junction(), 1);
    let mut commits = Commits::default();
    let minor = sim.spawn(&MINOR).expect("minor");
    commits.track(minor);
    let mut majors = Vec::new();
    let mut last_passed = None;
    while sim.tick() < 400 {
        if [80, 110, 140, 170, 200].contains(&sim.tick()) {
            majors.push(sim.spawn(&MAJOR).expect("major"));
        }
        commits.step(&mut sim);
        check_major_speed(&sim, &majors);
        if last_passed.is_none() && majors.len() == 5 && major_passed(&sim, majors[4]) {
            last_passed = Some(sim.tick());
        }
    }
    let tick = commits.tick_of(minor).expect("minor committed");
    assert!(
        last_passed.is_some_and(|passed| tick >= passed - 1),
        "{tick} {last_passed:?}"
    );
}

#[test]
fn stop_sign() {
    let mut sim = Sim::new(&with_control(t_junction(), Control::Stop), 1);
    let minor = sim.spawn(&MINOR).expect("minor");
    let mut stopped = false;
    for _ in 0..1_000 {
        let slot = slot_of(&sim, minor).expect("alive") as usize;
        let vehicles = sim.vehicles();
        stopped |= vehicles.stopped_at_line[slot];
        if vehicles.committed[slot] {
            assert!(stopped);
            return;
        }
        sim.step();
    }
    panic!("minor never committed");
}

#[test]
fn all_way_stop() {
    let mut sim = Sim::new(&with_control(four_way(2, 200.0), Control::AllWayStop), 1);
    let mut commits = Commits::default();
    let routes = [[0, 5], [2, 7], [4, 1], [6, 3]];
    let mut ids = Vec::new();
    while sim.tick() < 2_000 {
        if sim.tick().is_multiple_of(5) && ids.len() < routes.len() {
            let id = sim.spawn(&routes[ids.len()]).expect("spawn");
            commits.track(id);
            ids.push(id);
        }
        commits.step(&mut sim);
    }
    let ticks: Vec<u64> = ids
        .iter()
        .map(|&id| commits.tick_of(id).expect("committed"))
        .collect();
    assert!(ticks.windows(2).all(|pair| pair[0] <= pair[1]), "{ticks:?}");
    assert_eq!(sim.vehicle_count(), 0);
}

#[test]
fn timeout() {
    let mut sim = Sim::new(&t_junction(), 1);
    let mut committed_at = None;
    while sim.tick() < TIMEOUT_MINOR_TICK {
        spawn_major_every_minute(&mut sim);
        sim.step();
    }
    let minor = sim.spawn(&MINOR).expect("minor");
    while sim.tick() < 1_500 && committed_at.is_none() {
        spawn_major_every_minute(&mut sim);
        let slot = slot_of(&sim, minor).expect("alive") as usize;
        let waited = u64::from(sim.vehicles().wait_ticks[slot]);
        let tick = sim.tick();
        sim.step();
        check_invariants(&sim);
        if support::has_committed(&sim, minor) {
            assert!(
                waited >= WAIT_TIMEOUT_TICKS,
                "waited {waited} at tick {tick}"
            );
            committed_at = Some(tick);
        }
    }
    assert!(committed_at.is_some());
    for _ in 0..300 {
        sim.step();
        check_invariants(&sim);
    }
}

fn spawn_major_every_minute(sim: &mut Sim) {
    if sim.tick().is_multiple_of(60) {
        let _ = sim.spawn(&MAJOR);
    }
}

#[test]
fn blocked_leader_no_reservation() {
    let mut sim = Sim::new(&with_control(four_way(1, 200.0), Control::Priority), 1);
    let mut commits = Commits::default();
    let left = sim.spawn(&[0, 3]).expect("left");
    commits.track(left);
    let mut through = None;
    let mut cross = None;
    while sim.tick() < 3_000 {
        if through.is_none() {
            through = sim.spawn(&[0, 5]).ok();
            through.iter().for_each(|&id| commits.track(id));
        }
        if sim.tick() < 600 && sim.tick().is_multiple_of(30) {
            let _ = sim.spawn(&[4, 1]);
        }
        if sim.tick() == 100 {
            cross = sim.spawn(&[2, 7]).ok();
            cross.iter().for_each(|&id| commits.track(id));
        }
        commits.step(&mut sim);
        let through_tick = through.and_then(|id| commits.tick_of(id));
        if let Some(tick) = through_tick {
            assert!(commits.tick_of(left).is_some_and(|l| l <= tick));
        }
    }
    let cross_tick = cross
        .and_then(|id| commits.tick_of(id))
        .expect("cross committed");
    assert!(cross_tick < 600, "{cross_tick}");
    assert!(through.and_then(|id| commits.tick_of(id)).is_some());
}

#[test]
fn opposed_left_turns() {
    let mut sim = Sim::new(&with_control(four_way(1, 200.0), Control::Priority), 1);
    let mut ids = vec![
        sim.spawn(&[0, 3]).expect("left a"),
        sim.spawn(&[4, 7]).expect("left b"),
    ];
    while sim.tick() < 1_000 {
        if sim.tick() == 10 {
            ids.push(sim.spawn(&[0, 5]).expect("through a"));
            ids.push(sim.spawn(&[4, 1]).expect("through b"));
        }
        sim.step();
        check_invariants(&sim);
        if sim.tick() > 10 && ids.iter().all(|&id| slot_of(&sim, id).is_none()) {
            return;
        }
    }
    panic!("vehicles still live after 1000 ticks");
}
