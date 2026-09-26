mod support;

use support::{run_until, slot_of};
use trapiks_sim_core::consts::IDM_MIN_GAP;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::entry::has_space;

#[test]
fn spillback() {
    let mut sim = Sim::new(&four_way(1, 200.0), 1);
    let a = sim.spawn(&[3]).expect("a");
    let b = sim.spawn(&[0, 3]).expect("b");
    let b_slot = slot_of(&sim, b).expect("b alive");
    assert!(!has_space(&sim.rule_context(), b_slot));
    let span_start = sim.network().link_span(3).0;
    let cleared = run_until(&mut sim, 200, |sim| {
        slot_of(sim, a).is_some_and(|slot| {
            let vehicles = sim.vehicles();
            let free = vehicles.s[slot as usize] - vehicles.length(slot) - span_start;
            free >= vehicles.length(b_slot) + IDM_MIN_GAP
        })
    });
    assert!(cleared);
    sim.step();
    assert!(has_space(&sim.rule_context(), b_slot));
}
