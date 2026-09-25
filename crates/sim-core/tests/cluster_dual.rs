use trapiks_sim_core::fixtures::dual_carriageway_cross;
use trapiks_sim_core::network::{Network, SignalState};

#[test]
fn box_nodes_form_one_cluster() {
    let network = Network::from_map(&dual_carriageway_cross());
    let clusters = network.signals().clusters();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].nodes, vec![0, 1, 2, 3]);
    assert_eq!(clusters[0].approaches, vec![0, 6, 12, 18]);
    assert_eq!(clusters[0].phases, vec![vec![0, 6], vec![12, 18]]);
    assert_eq!(network.signal_state(0, 0), Some(SignalState::Green));
}

#[test]
fn internal_links_are_never_stopped() {
    let network = Network::from_map(&dual_carriageway_cross());
    for link in [2, 8, 14, 20] {
        assert!(network.signals().is_internal(link));
        for tick in [0, 300, 330, 350, 650] {
            assert_eq!(network.signal_state(link, tick), None);
        }
    }
}
