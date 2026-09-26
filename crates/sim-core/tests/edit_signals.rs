mod support;

use support::{apply_now, slot_of};
use trapiks_sim_core::edit::{EditCommand, Outcome};
use trapiks_sim_core::fixtures::{four_way, t_junction};
use trapiks_sim_core::map::Control;
use trapiks_sim_core::network::{SignalState, cycle_ticks};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const MINOR_APPROACH: u32 = 4;
const MINOR_ROUTE: [u32; 2] = [4, 2];

fn is_ok(outcome: &Outcome) -> bool {
    matches!(outcome, Outcome::Ok(_))
}

fn has_entered(sim: &Sim, id: u32) -> bool {
    let Some(slot) = slot_of(sim, id) else {
        return true;
    };
    let index = slot as usize;
    let vehicles = sim.vehicles();
    vehicles.committed[index] || matches!(vehicles.place[index], Place::Movement { .. })
}

fn is_waiting(sim: &Sim, id: u32) -> bool {
    slot_of(sim, id).is_some_and(|slot| sim.vehicles().v[slot as usize] < 0.5)
}

#[test]
fn control_edit() {
    let mut sim = Sim::new(&t_junction(), 1);
    let signal = EditCommand::SetJunctionControl {
        node: 0,
        control: Control::Signal,
    };
    assert!(is_ok(&apply_now(&mut sim, signal)));
    assert!(sim.network().signals().cluster_of(0).is_some());
    let id = sim.spawn(&MINOR_ROUTE).expect("spawn");
    let mut waited_on_red = false;
    while !has_entered(&sim, id) && sim.tick() < 2_000 {
        let state = sim.network().signal_state(MINOR_APPROACH, sim.tick());
        waited_on_red |= state == Some(SignalState::Red) && is_waiting(&sim, id);
        sim.step();
    }
    assert!(waited_on_red);
    let entered_at = sim.tick() - 1;
    let state = sim.network().signal_state(MINOR_APPROACH, entered_at);
    assert_eq!(state, Some(SignalState::Green));
    assert!(is_ok(&apply_now(&mut sim, EditCommand::Undo)));
    assert_eq!(sim.network().nodes.control[0], Control::Priority);
    assert!(sim.network().signals().cluster_of(0).is_none());
}

fn phase_states(sim: &Sim, approach: u32, ticks: std::ops::Range<u64>) -> Vec<Option<SignalState>> {
    ticks
        .map(|tick| sim.network().signal_state(approach, tick))
        .collect()
}

fn all_are(states: &[Option<SignalState>], expected: SignalState) -> bool {
    states.iter().all(|&state| state == Some(expected))
}

fn cluster_cycle(sim: &Sim) -> u64 {
    sim.network()
        .signals()
        .cluster_at(0)
        .map_or(0, |cluster| cycle_ticks(&cluster.green_ticks))
}

fn assert_short_cycle(sim: &Sim, first: u32, second: u32) {
    let states = phase_states(sim, first, 0..200);
    assert!(all_are(&states[..50], SignalState::Green));
    assert!(all_are(&states[50..80], SignalState::Amber));
    assert!(all_are(&states[80..], SignalState::Red));
    let states = phase_states(sim, second, 0..200);
    assert!(all_are(&states[..100], SignalState::Red));
    assert!(all_are(&states[100..150], SignalState::Green));
}

#[test]
fn timing_edit() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    assert_eq!(cluster_cycle(&sim), 700);
    let cluster = sim.network().signals().cluster_at(0).expect("cluster");
    let (first, second) = (cluster.phases[0][0], cluster.phases[1][0]);
    let key = cluster.key();
    let timing = EditCommand::SetSignalTiming {
        node: 0,
        greens_s: vec![5, 5],
        offset_s: 0,
    };
    assert!(is_ok(&apply_now(&mut sim, timing)));
    assert_eq!(cluster_cycle(&sim), 200);
    assert_short_cycle(&sim, first, second);
    assert!(sim.network().timing_override(key).is_some());
    assert!(is_ok(&apply_now(&mut sim, EditCommand::Undo)));
    assert_eq!(cluster_cycle(&sim), 700);
    assert!(sim.network().timing_override(key).is_none());
}
