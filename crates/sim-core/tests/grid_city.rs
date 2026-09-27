use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::{AreaKind, Control, from_bytes, to_bytes, validate};

const SPEC: GridCity = GridCity {
    cols: 10,
    rows: 10,
    spacing: 150.0,
};

#[test]
fn grid_city_is_valid_and_deterministic() {
    let map = grid_city(&SPEC);
    assert_eq!(validate(&map), Ok(()));
    assert_eq!(map, grid_city(&SPEC));
    assert_eq!(map.node_count(), 11 * 11 + 3);
    assert_eq!(map.road_count(), 11 * 10 + 11 * 10 + 4);
    assert_eq!(map.node_count(), 124);
    assert_eq!(map.road_count(), 224);
    let kinds = [
        AreaKind::Water,
        AreaKind::Water,
        AreaKind::Park,
        AreaKind::Park,
    ];
    assert_eq!(map.areas.kind, kinds.to_vec());
    assert_eq!(map.areas.hole, vec![false, false, false, true]);
    let bytes = to_bytes(&map).unwrap_or_default();
    assert_eq!(from_bytes(&bytes), Ok(map));
}

#[test]
fn grid_city_layout() {
    let map = grid_city(&SPEC);
    let signals = map.nodes.control.iter().filter(|&&c| c == Control::Signal);
    assert_eq!(signals.count(), 9);
    assert_eq!(map.names[0], "");
    assert!(map.names.iter().any(|name| name == "Skyway"));
    let one_way = (0..map.road_count()).filter(|&r| map.roads.lanes_backward[r] == 0);
    assert_eq!(one_way.count(), 20 + 4);
}
