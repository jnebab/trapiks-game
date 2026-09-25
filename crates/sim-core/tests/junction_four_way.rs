use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::{Junction, Network, TurnKind};

fn centre() -> (Network, Junction) {
    let mut network = Network::from_map(&four_way(2, 200.0));
    let junction = network.ensure_junction(0).clone();
    (network, junction)
}

fn crossings(junction: &Junction, i: usize, j: usize) -> usize {
    junction.conflicts[i]
        .iter()
        .filter(|c| usize::from(c.other) == j && !c.merge)
        .count()
}

fn through_from(junction: &Junction, from: u32) -> usize {
    junction
        .movements
        .iter()
        .position(|m| m.from_link == from && m.kind == TurnKind::Through)
        .unwrap_or(usize::MAX)
}

#[test]
fn twelve_movements_without_uturns() {
    let (_, junction) = centre();
    assert_eq!(junction.movements.len(), 12);
    assert!(junction.movements.iter().all(|m| m.kind != TurnKind::UTurn));
}

#[test]
fn lane_ranges_follow_turn_kind() {
    let (_, junction) = centre();
    for m in &junction.movements {
        let expected = match m.kind {
            TurnKind::Through => (0, 1),
            TurnKind::Right => (0, 0),
            TurnKind::Left | TurnKind::UTurn => (1, 1),
        };
        assert_eq!(m.from_lanes, expected, "{m:?}");
    }
}

#[test]
fn opposing_throughs_do_not_conflict_and_perpendicular_cross_once() {
    let (_, junction) = centre();
    let north = through_from(&junction, 0);
    let east = through_from(&junction, 2);
    let south = through_from(&junction, 4);
    assert_eq!(crossings(&junction, north, south), 0);
    assert!(
        junction.conflicts[north]
            .iter()
            .all(|c| usize::from(c.other) != south)
    );
    assert_eq!(crossings(&junction, north, east), 1);
    assert_eq!(crossings(&junction, east, north), 1);
}

#[test]
fn shared_targets_merge() {
    let (_, junction) = centre();
    for (i, a) in junction.movements.iter().enumerate() {
        for (j, b) in junction.movements.iter().enumerate() {
            if i == j || a.to_link != b.to_link {
                continue;
            }
            let merge = junction.conflicts[i]
                .iter()
                .find(|c| usize::from(c.other) == j)
                .expect("merge conflict");
            assert!(merge.merge);
            assert_eq!((merge.s_self, merge.s_other), (a.length, b.length));
        }
    }
}

#[test]
fn setback_is_half_width_plus_margin() {
    let (network, _) = centre();
    assert!((network.setback_at(0, 0) - 7.4).abs() < 1e-9);
    let (start, end) = network.link_span(0);
    assert_eq!(start, 0.0);
    assert!((end - (200.0 - 7.4)).abs() < 1e-9);
}
