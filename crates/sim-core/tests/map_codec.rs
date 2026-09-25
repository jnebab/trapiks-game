use trapiks_sim_core::MAP_FORMAT_VERSION;
use trapiks_sim_core::fnv::fnv1a64;
use trapiks_sim_core::map::{
    AreaKind, AreaTable, Control, GeoOrigin, MapData, MapError, NodeTable, PointTable, RoadClass,
    RoadTable, TurnBan, from_bytes, map_hash, to_bytes,
};

fn sample_map() -> MapData {
    MapData {
        origin: GeoOrigin {
            lat: 14.6,
            lon: 121.0,
        },
        nodes: NodeTable {
            x: vec![0.0, 10.0, 20.0],
            y: vec![0.0, 0.0, 5.0],
            control: vec![Control::Priority, Control::Signal, Control::Yield],
        },
        roads: RoadTable {
            from: vec![0, 1],
            to: vec![1, 2],
            class: vec![RoadClass::Residential, RoadClass::Primary],
            lanes_forward: vec![1, 2],
            lanes_backward: vec![1, 0],
            speed_kph: vec![20, 60],
            layer: vec![0, 1],
            name: vec![0, 1],
            point_start: vec![0, 2, 5],
        },
        points: PointTable {
            x: vec![0.0, 10.0, 10.0, 15.0, 20.0],
            y: vec![0.0, 0.0, 0.0, 2.0, 5.0],
        },
        names: vec![String::new(), "EDSA".to_string()],
        turn_bans: vec![TurnBan {
            via_node: 1,
            from_road: 0,
            to_road: 1,
        }],
        areas: AreaTable {
            kind: vec![AreaKind::Water],
            ring_start: vec![0, 3],
            x: vec![0.0, 10.0, 0.0],
            y: vec![0.0, 0.0, 10.0],
        },
    }
}

fn encode(map: &MapData) -> Vec<u8> {
    to_bytes(map).unwrap_or_default()
}

#[test]
fn round_trips_a_valid_map() {
    let map = sample_map();
    assert_eq!(from_bytes(&encode(&map)), Ok(map));
}

#[test]
fn rejects_bad_magic() {
    let mut bytes = encode(&sample_map());
    bytes[0] = b'X';
    assert_eq!(from_bytes(&bytes), Err(MapError::BadMagic));
}

#[test]
fn rejects_unsupported_version() {
    let mut bytes = encode(&sample_map());
    bytes[4..8].copy_from_slice(&99u32.to_le_bytes());
    assert_eq!(from_bytes(&bytes), Err(MapError::UnsupportedVersion(99)));
    assert_ne!(MAP_FORMAT_VERSION, 99);
}

#[test]
fn rejects_road_with_missing_node() {
    let mut map = sample_map();
    map.roads.to[1] = 7;
    assert!(matches!(
        from_bytes(&encode(&map)),
        Err(MapError::Invalid(_))
    ));
}

#[test]
fn map_hash_is_stable_for_equal_bytes() {
    let first = encode(&sample_map());
    let second = encode(&sample_map());
    assert_eq!(map_hash(&first), map_hash(&second));
    assert_eq!(road_points_len(&sample_map()), 3);
}

fn road_points_len(map: &MapData) -> usize {
    map.road_points(1).0.len()
}

#[test]
fn fnv1a64_matches_reference_vectors() {
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn rejects_area_ring_with_two_points() {
    let mut map = sample_map();
    map.areas.ring_start = vec![0, 2];
    map.areas.x.truncate(2);
    map.areas.y.truncate(2);
    assert!(matches!(
        from_bytes(&encode(&map)),
        Err(MapError::Invalid(_))
    ));
}

#[test]
fn rejects_bad_ring_start() {
    let mut map = sample_map();
    map.areas.ring_start = vec![1, 3];
    assert!(matches!(
        from_bytes(&encode(&map)),
        Err(MapError::Invalid(_))
    ));
    map.areas.ring_start = vec![0];
    assert!(matches!(
        from_bytes(&encode(&map)),
        Err(MapError::Invalid(_))
    ));
}

#[test]
fn area_ring_returns_ring_points() {
    let map = sample_map();
    assert_eq!(map.area_ring(0).0, &[0.0, 10.0, 0.0]);
}
