use trapiks_sim_core::edit::EditCommand;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::sim::Sim;

fn centre_sim() -> Sim {
    Sim::new(&four_way(2, 200.0), 1)
}

#[test]
fn four_way_lists_all_turns_and_two_phases() {
    let sim = centre_sim();
    let info = sim.inspect_node(0).expect("node");
    assert_eq!(info.turns.len(), 12);
    assert!(info.turns.iter().all(|turn| turn.allowed));
    assert_eq!(info.signal.expect("signal").phase_count, 2);
}

#[test]
fn four_way_offers_both_straight_flyovers() {
    let sim = centre_sim();
    let info = sim.inspect_node(0).expect("node");
    assert_eq!(info.flyover_pairs, vec![[0, 2], [1, 3]]);
}

#[test]
fn banned_turn_is_reported_as_not_allowed() {
    let mut sim = centre_sim();
    let turn = sim.inspect_node(0).expect("node").turns[0].clone();
    sim.enqueue(EditCommand::SetTurnAllowed {
        node: 0,
        from_road: turn.from_road,
        to_road: turn.to_road,
        allowed: false,
    });
    sim.step();
    let after = sim.inspect_node(0).expect("node");
    let banned: Vec<_> = after.turns.iter().filter(|t| !t.allowed).collect();
    assert_eq!(banned.len(), 1);
    assert_eq!(
        (banned[0].from_road, banned[0].to_road),
        (turn.from_road, turn.to_road)
    );
}

#[test]
fn road_inspection_reports_lanes() {
    let sim = centre_sim();
    let info = sim.inspect_road(1).expect("road");
    assert_eq!((info.lanes_forward, info.lanes_backward), (2, 2));
    assert!(sim.inspect_road(99).is_none());
}
