use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::{Network, SignalState};

const NORTH: u32 = 0;
const EAST: u32 = 2;

#[test]
fn four_way_has_one_two_phase_cluster() {
    let network = Network::from_map(&four_way(2, 200.0));
    let clusters = network.signals().clusters();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].approaches, vec![0, 2, 4, 6]);
    assert_eq!(clusters[0].phases, vec![vec![0, 4], vec![2, 6]]);
    assert_eq!(network.signals().cluster_of(0), Some(0));
}

#[test]
fn phases_cycle_through_green_amber_red() {
    let network = Network::from_map(&four_way(2, 200.0));
    let state = |link, tick| network.signal_state(link, tick);
    assert_eq!(state(NORTH, 0), Some(SignalState::Green));
    assert_eq!(state(EAST, 0), Some(SignalState::Red));
    assert_eq!(state(NORTH, 300), Some(SignalState::Amber));
    assert_eq!(state(NORTH, 330), Some(SignalState::Red));
    assert_eq!(state(EAST, 330), Some(SignalState::Red));
    assert_eq!(state(EAST, 350), Some(SignalState::Green));
    assert_eq!(state(NORTH, 700), Some(SignalState::Green));
    assert_eq!(state(NORTH, 699), Some(SignalState::Red));
    assert_eq!(state(1, 0), None);
}
