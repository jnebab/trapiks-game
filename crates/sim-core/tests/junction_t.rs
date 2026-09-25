use trapiks_sim_core::fixtures::t_junction;
use trapiks_sim_core::network::{Network, TurnKind};

const MINOR: u32 = 4;

#[test]
fn minor_approach_turns_right_and_left_from_its_only_lane() {
    let mut network = Network::from_map(&t_junction());
    let junction = network.ensure_junction(0);
    let minor: Vec<_> = junction
        .movements
        .iter()
        .filter(|m| m.from_link == MINOR)
        .collect();
    let kinds: Vec<TurnKind> = minor.iter().map(|m| m.kind).collect();
    assert_eq!(kinds, vec![TurnKind::Left, TurnKind::Right]);
    assert!(minor.iter().all(|m| m.from_lanes == (0, 0)));
}

#[test]
fn major_movements_outrank_minor_ones() {
    let mut network = Network::from_map(&t_junction());
    let junction = network.ensure_junction(0);
    let (minor, major): (Vec<_>, Vec<_>) = junction
        .movements
        .iter()
        .partition(|m| m.from_link == MINOR);
    let best_minor = minor.iter().map(|m| m.rank).max().expect("minor");
    assert!(major.iter().all(|m| m.rank > best_minor));
}

#[test]
fn no_uturns_at_the_junction() {
    let mut network = Network::from_map(&t_junction());
    let junction = network.ensure_junction(0);
    assert!(junction.movements.iter().all(|m| m.kind != TurnKind::UTurn));
}
