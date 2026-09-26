mod support;

use support::{Commits, check_invariants, slot_of};
use trapiks_sim_core::consts::STOPPED_SPEED;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

fn step_to(sim: &mut Sim, commits: &mut Commits, tick: u64) {
    while sim.tick() < tick {
        commits.step(sim);
    }
}

fn link_state(sim: &Sim, id: u32) -> Option<(u32, f64, f64)> {
    let slot = slot_of(sim, id)? as usize;
    let vehicles = sim.vehicles();
    match vehicles.place[slot] {
        Place::Link { link, .. } => Some((link, vehicles.s[slot], vehicles.v[slot])),
        Place::Movement { .. } => None,
    }
}

#[test]
fn signal_red_stop() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let id = sim.spawn(&[2, 7]).expect("spawn");
    let span_end = sim.network().link_span(2).1;
    let mut commits = Commits::default();
    commits.track(id);
    step_to(&mut sim, &mut commits, 349);
    let (link, s, v) = link_state(&sim, id).expect("on link");
    assert_eq!(link, 2);
    assert!(s <= span_end);
    assert!(v < STOPPED_SPEED);
    step_to(&mut sim, &mut commits, 400);
    let tick = commits.tick_of(id).expect("committed");
    assert!((350..360).contains(&tick), "{tick}");
}

#[test]
fn signal_amber() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let mut commits = Commits::default();
    step_to(&mut sim, &mut commits, 190);
    let first = sim.spawn(&[0, 5]).expect("spawn");
    commits.track(first);
    step_to(&mut sim, &mut commits, 250);
    let second = sim.spawn(&[0, 5]).expect("spawn");
    commits.track(second);
    step_to(&mut sim, &mut commits, 300);
    let first_tick = commits.tick_of(first).expect("first committed");
    assert!(
        first_tick < 300 && first_tick.abs_diff(261) <= 5,
        "{first_tick}"
    );
    let second_slot = slot_of(&sim, second).expect("alive") as usize;
    assert_eq!(sim.vehicles().arrival_tick[second_slot], u64::MAX);
    step_to(&mut sim, &mut commits, 330);
    let first_slot = slot_of(&sim, first).expect("alive") as usize;
    assert!(sim.vehicles().route_cursor[first_slot] >= 1);
    step_to(&mut sim, &mut commits, 450);
    let (link, s, v) = link_state(&sim, second).expect("second on link");
    assert_eq!(link, 0);
    assert!(v < STOPPED_SPEED);
    assert!(s <= sim.network().link_span(0).1);
    step_to(&mut sim, &mut commits, 750);
    let second_tick = commits.tick_of(second).expect("second committed");
    assert!((700..710).contains(&second_tick), "{second_tick}");
}

#[test]
fn permissive_left() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let mut commits = Commits::default();
    let left = sim.spawn(&[0, 3]).expect("spawn");
    commits.track(left);
    while sim.tick() < 1_400 {
        if sim.tick() < 400 && sim.tick().is_multiple_of(30) {
            let _ = sim.spawn(&[4, 1]);
        }
        commits.step(&mut sim);
        check_invariants(&sim);
    }
    assert!(commits.tick_of(left).is_some());
}
