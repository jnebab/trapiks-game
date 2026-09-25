use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::Network;

#[test]
fn a_ban_removes_exactly_one_movement() {
    let mut map = four_way(2, 200.0);
    let mut open = Network::from_map(&map);
    let before: Vec<(u32, u32)> = pairs(open.ensure_junction(0));
    map.turn_bans.push(trapiks_sim_core::map::TurnBan {
        via_node: 0,
        from_road: 0,
        to_road: 1,
    });
    let mut banned = Network::from_map(&map);
    assert!(banned.is_banned(0, 0, 1));
    let after = pairs(banned.ensure_junction(0));
    let removed: Vec<_> = before.iter().filter(|p| !after.contains(p)).collect();
    assert_eq!(removed, vec![&(0, 3)]);
    assert_eq!(after.len(), before.len() - 1);
}

fn pairs(junction: &trapiks_sim_core::network::Junction) -> Vec<(u32, u32)> {
    junction
        .movements
        .iter()
        .map(|m| (m.from_link, m.to_link))
        .collect()
}
