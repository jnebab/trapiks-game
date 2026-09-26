use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::Network;
use trapiks_sim_core::render::{PILL_AMBER, PILL_GREEN, PILL_RED, signal_pills, signal_states};

fn states_at(network: &Network, links: &[u32], tick: u64) -> Vec<u8> {
    let mut out = Vec::new();
    signal_states(network, links, tick, &mut out);
    out
}

#[test]
fn four_way_has_four_pills() {
    let network = Network::from_map(&four_way(2, 200.0));
    let pills = signal_pills(&network);
    assert_eq!(pills.link, vec![0, 2, 4, 6]);
    assert_eq!(pills.x.len(), 4);
    assert_eq!(pills.angle.len(), 4);
}

#[test]
fn pill_states_follow_signal_windows() {
    let network = Network::from_map(&four_way(2, 200.0));
    let links = signal_pills(&network).link;
    let (g, a, r) = (PILL_GREEN, PILL_AMBER, PILL_RED);
    assert_eq!(states_at(&network, &links, 0), vec![g, r, g, r]);
    assert_eq!(states_at(&network, &links, 300), vec![a, r, a, r]);
    assert_eq!(states_at(&network, &links, 330), vec![r, r, r, r]);
    assert_eq!(states_at(&network, &links, 350), vec![r, g, r, g]);
}
