use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::{AreaKind, Control, MapData, RoadClass, to_bytes};
use trapiks_sim_core::map_meta;
use trapiks_sim_core::render::{area_render, node_render, road_render};

const SPEC: GridCity = GridCity {
    cols: 4,
    rows: 3,
    spacing: 100.0,
};

#[test]
fn road_render_columns_match_map_lengths() {
    let map = grid_city(&SPEC);
    let roads = road_render(&map);
    assert_eq!(roads.point_start.len(), map.road_count() + 1);
    assert_eq!(roads.x.len(), map.points.x.len());
    assert_eq!(roads.y.len(), map.points.y.len());
    assert_eq!(roads.class.len(), map.road_count());
    assert_eq!(roads.lanes_forward.len(), map.road_count());
    assert_eq!(roads.lanes_backward.len(), map.road_count());
    assert_eq!(roads.layer.len(), map.road_count());
    assert_eq!(roads.name.len(), map.road_count());
}

#[test]
fn node_render_columns_match_map_lengths() {
    let map = grid_city(&SPEC);
    let nodes = node_render(&map);
    assert_eq!(nodes.x.len(), map.node_count());
    assert_eq!(nodes.y.len(), map.node_count());
    assert_eq!(nodes.control.len(), map.node_count());
}

#[test]
fn area_render_columns_match_map_lengths() {
    let map = grid_city(&SPEC);
    let areas = area_render(&map);
    assert_eq!(areas.kind.len(), map.areas.kind.len());
    assert_eq!(areas.ring_start, map.areas.ring_start);
    assert_eq!(areas.x.len(), map.areas.x.len());
    assert_eq!(areas.y.len(), map.areas.y.len());
}

#[test]
fn codes_round_trip() {
    for class in RoadClass::ALL {
        assert_eq!(RoadClass::ALL[usize::from(class.code())], class);
    }
    for control in Control::ALL {
        assert_eq!(Control::ALL[usize::from(control.code())], control);
    }
    for kind in AreaKind::ALL {
        assert_eq!(AreaKind::ALL[usize::from(kind.code())], kind);
    }
}

#[test]
fn map_meta_reports_bounds_and_hash() {
    let map = grid_city(&SPEC);
    let bytes = to_bytes(&map).unwrap_or_default();
    let meta = map_meta(&map, &bytes);
    assert_eq!(meta.map_hash.len(), 16);
    assert!(
        meta.map_hash
            .chars()
            .all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    );
    assert_eq!(meta.road_count as usize, map.road_count());
    assert_eq!(meta.class_names.len(), 15);
    let min_x = map.points.x.iter().copied().fold(f32::INFINITY, f32::min);
    let max_y = map
        .points
        .y
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    assert_eq!(meta.bounds[0], min_x);
    assert_eq!(meta.bounds[3], max_y);
    assert!(meta.bounds[0] < meta.bounds[2]);
    assert!(meta.bounds[1] < meta.bounds[3]);
}

#[test]
fn map_meta_bounds_are_zero_without_points() {
    let meta = map_meta(&MapData::default(), &[]);
    assert_eq!(meta.bounds, [0.0; 4]);
    assert_eq!(meta.map_hash.len(), 16);
}
