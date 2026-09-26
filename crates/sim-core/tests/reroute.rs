mod support;

use support::slot_of;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::{Network, reverse};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

fn remaining_route(sim: &Sim, slot: u32) -> Vec<u32> {
    let vehicles = sim.vehicles();
    vehicles.route(slot)[vehicles.cursor(slot)..].to_vec()
}

fn assert_valid_chain(route: &[u32]) {
    let mut map = four_way(2, 200.0);
    map.turn_bans.push(trapiks_sim_core::map::TurnBan {
        via_node: 0,
        from_road: 0,
        to_road: 1,
    });
    let mut network = Network::from_map(&map);
    for pair in route.windows(2) {
        let junction = network.ensure_junction(network.link_to(pair[0]));
        assert!(
            junction.movement_index(pair[0], pair[1]).is_some(),
            "{route:?}"
        );
    }
}

#[test]
fn reroute_on_invalid() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let id = sim.spawn(&[0, 3]).expect("spawn");
    sim.network_mut().ban_turn_for_test(0, 0, 1);
    sim.step();
    let slot = slot_of(&sim, id).expect("alive");
    let route = remaining_route(&sim, slot);
    assert!(route.windows(2).all(|pair| pair != [0, 3]), "{route:?}");
    assert!(route.windows(2).any(|pair| pair[1] == reverse(pair[0])));
    assert_eq!(route.last(), Some(&3));
    assert_valid_chain(&route);
    let mut reached_end = false;
    while slot_of(&sim, id).is_some() && sim.tick() < 5_000 {
        let vehicles = sim.vehicles();
        let s = vehicles.s[slot as usize];
        if vehicles.place[slot as usize] == (Place::Link { link: 3, lane: 0 })
            || matches!(vehicles.place[slot as usize], Place::Link { link: 3, .. })
        {
            reached_end |= s >= sim.network().link_span(3).1 - 5.0;
        }
        sim.step();
    }
    assert!(reached_end);
    assert!(slot_of(&sim, id).is_none());
    assert_eq!(sim.stranded(), 0);
}

#[test]
fn strands_without_exit() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let id = sim.spawn(&[0, 3]).expect("spawn");
    for to_road in 1..4 {
        sim.network_mut().ban_turn_for_test(0, 0, to_road);
    }
    sim.step();
    assert!(slot_of(&sim, id).is_none());
    assert_eq!(sim.stranded(), 1);
}
