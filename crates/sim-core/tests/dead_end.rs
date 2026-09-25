use trapiks_sim_core::fixtures::dead_end;
use trapiks_sim_core::network::{Network, TurnKind};

#[test]
fn dead_end_has_a_single_uturn() {
    let mut network = Network::from_map(&dead_end());
    let junction = network.ensure_junction(3);
    assert_eq!(junction.movements.len(), 1);
    assert_eq!(junction.movements[0].kind, TurnKind::UTurn);
    assert_eq!(junction.movements[0].from_link, 4);
    assert_eq!(junction.movements[0].to_link, 5);
}

#[test]
fn junction_node_has_no_uturns() {
    let mut network = Network::from_map(&dead_end());
    let junction = network.ensure_junction(0);
    assert!(junction.movements.iter().all(|m| m.kind != TurnKind::UTurn));
}
