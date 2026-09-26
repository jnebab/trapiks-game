mod support;

use support::{Fixture, LAT, LON, STEP, four_way, restriction, single_way_map};
use trapiks_mapgen::input::{LoadStats, OsmData, merge, parse};
use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::map::{Control, MapData, validate};

const OVERPASS_JSON: &str = r#"{
  "elements": [
    { "type": "way", "id": 10, "nodes": [1, 2],
      "geometry": [{ "lat": 14.6, "lon": 121.0 }, { "lat": 14.6, "lon": 121.0001 }],
      "tags": { "highway": "residential" } },
    { "type": "way", "id": 11, "nodes": [2, 3],
      "geometry": [{ "lat": 14.6, "lon": 121.0001 }],
      "tags": { "highway": "residential" } },
    { "type": "node", "id": 1, "lat": 14.6, "lon": 121.0,
      "tags": { "highway": "traffic_signals" } },
    { "type": "relation", "id": 20,
      "members": [{ "type": "way", "ref": 10, "role": "from" }],
      "tags": { "type": "restriction", "restriction": "no_u_turn" } }
  ]
}"#;

fn road_signature(map: &MapData) -> Vec<(u8, u8)> {
    (0..map.road_count())
        .map(|r| (map.roads.lanes_forward[r], map.roads.lanes_backward[r]))
        .collect()
}

fn lanes_of(pairs: &[(&str, &str)]) -> (u8, u8) {
    road_signature(&single_way_map(pairs))[0]
}

#[test]
fn parses_overpass_json() {
    let mut osm = OsmData::default();
    let mut stats = LoadStats::default();
    match parse(OVERPASS_JSON.as_bytes()) {
        Ok(response) => merge(&mut osm, &mut stats, response),
        Err(error) => panic!("parse failed: {error}"),
    }
    assert_eq!(stats.dropped_ways, 1);
    assert_eq!(osm.ways.len(), 1);
    assert_eq!(osm.relations[&20].members[0].id, 10);
    assert_eq!(osm.relations[&20].members[0].kind, "way");
    assert_eq!(osm.nodes[&1].tags["highway"], "traffic_signals");
    assert_eq!(osm.nodes[&2].lon, 121.0001);
}

#[test]
fn loads_fixture_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tiny.json").to_string();
    let loaded = trapiks_mapgen::input::load(&[path]);
    let Ok((osm, _)) = loaded else {
        panic!("load failed");
    };
    let map = Fixture { osm }.build();
    assert_eq!(map.road_count(), 4);
    assert_eq!(map.turn_bans.len(), 1);
    assert!(map.nodes.control.contains(&Control::Signal));
}

#[test]
fn splits_crossing_ways() {
    let mut fixture = Fixture::default();
    fixture
        .node(1, LAT, LON - STEP)
        .node(2, LAT, LON)
        .node(3, LAT, LON + STEP)
        .node(4, LAT - STEP, LON)
        .node(5, LAT + STEP, LON)
        .way(10, &[1, 2, 3], &[("highway", "residential")])
        .way(11, &[4, 2, 5], &[("highway", "residential")]);
    let map = fixture.build();
    assert_eq!(map.road_count(), 4);
    assert_eq!(map.node_count(), 5);
}

fn chain(middle: &[i64], outer: &[(&str, &str)], inner: &[(&str, &str)]) -> MapData {
    let mut fixture = Fixture::default();
    fixture
        .row(1, 6)
        .way(10, &[1, 2, 3], outer)
        .way(11, middle, inner)
        .way(12, &[4, 5, 6], outer);
    fixture.build()
}

#[test]
fn merges_equal_chain() {
    let street = [("highway", "residential"), ("name", "Mabini")];
    let wide = [
        ("highway", "residential"),
        ("name", "Mabini"),
        ("lanes", "4"),
    ];
    assert_eq!(chain(&[3, 4], &street, &street).road_count(), 1);
    assert_eq!(chain(&[3, 4], &street, &wide).road_count(), 3);
    let outer = [
        ("highway", "residential"),
        ("lanes:forward", "2"),
        ("lanes:backward", "1"),
    ];
    let inner = [
        ("highway", "residential"),
        ("lanes:forward", "1"),
        ("lanes:backward", "2"),
    ];
    let reversed = chain(&[4, 3], &outer, &inner);
    assert_eq!(reversed.road_count(), 1);
    assert_eq!(reversed.roads.point_start, vec![0, 6]);
    let signature = road_signature(&reversed)[0];
    let from_first = reversed.roads.from[0] == 0;
    assert_eq!(signature, if from_first { (2, 1) } else { (1, 2) });
}

#[test]
fn oneway_variants() {
    let forward = single_way_map(&[("highway", "residential"), ("oneway", "yes")]);
    assert_eq!(road_signature(&forward), vec![(1, 0)]);
    assert!(
        forward.nodes.x[forward.roads.from[0] as usize]
            < forward.nodes.x[forward.roads.to[0] as usize]
    );
    let backward = single_way_map(&[("highway", "residential"), ("oneway", "-1")]);
    assert_eq!(road_signature(&backward), vec![(1, 0)]);
    assert!(
        backward.nodes.x[backward.roads.from[0] as usize]
            > backward.nodes.x[backward.roads.to[0] as usize]
    );
    let roundabout = single_way_map(&[("highway", "residential"), ("junction", "roundabout")]);
    assert_eq!(road_signature(&roundabout), vec![(1, 0)]);
    assert_eq!(lanes_of(&[("highway", "motorway")]), (2, 0));
    assert_eq!(lanes_of(&[("highway", "motorway_link")]), (1, 1));
    let oneway = [("highway", "residential"), ("oneway", "yes")];
    let mut fixture = Fixture::default();
    fixture
        .row(1, 3)
        .way(10, &[1, 2], &oneway)
        .way(11, &[3, 2], &oneway);
    assert_eq!(fixture.build().road_count(), 2);
}

#[test]
fn lane_rules() {
    let with = |pairs: &[(&str, &str)]| {
        let mut all = vec![("highway", "residential")];
        all.extend_from_slice(pairs);
        lanes_of(&all)
    };
    assert_eq!(with(&[("lanes", "4")]), (2, 2));
    assert_eq!(with(&[("lanes", "3")]), (2, 1));
    assert_eq!(
        with(&[("lanes:forward", "3"), ("lanes:backward", "1")]),
        (3, 1)
    );
    assert_eq!(with(&[("lanes", "3"), ("lanes:backward", "2")]), (1, 2));
    assert_eq!(with(&[]), (1, 1));
    assert_eq!(with(&[("oneway", "yes"), ("lanes", "3")]), (3, 0));
}

fn speed_of(maxspeed: &str) -> u8 {
    single_way_map(&[("highway", "primary"), ("maxspeed", maxspeed)])
        .roads
        .speed_kph[0]
}

#[test]
fn speed_rules() {
    assert_eq!(speed_of("60"), 60);
    assert_eq!(speed_of("60 km/h"), 60);
    assert_eq!(speed_of("40 mph"), 64);
    assert_eq!(speed_of("none"), 60);
    assert_eq!(speed_of("3000000 mph"), 60);
    assert_eq!(speed_of("65535 mph"), 130);
}

fn layer_of(key: &str, value: &str) -> i8 {
    single_way_map(&[("highway", "residential"), (key, value)])
        .roads
        .layer[0]
}

#[test]
fn layer_rules() {
    assert_eq!(layer_of("layer", "2"), 2);
    assert_eq!(layer_of("bridge", "yes"), 1);
    assert_eq!(layer_of("tunnel", "yes"), -1);
    assert_eq!(layer_of("layer", "9"), 5);
}

#[test]
fn filters() {
    use trapiks_mapgen::filter::road_class;
    let tags = |pairs: &[(&str, &str)]| {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    };
    assert!(road_class(&tags(&[("highway", "service"), ("service", "driveway")])).is_none());
    assert!(road_class(&tags(&[("highway", "residential"), ("access", "private")])).is_none());
    assert!(road_class(&tags(&[("highway", "residential"), ("area", "yes")])).is_none());
    assert!(road_class(&tags(&[("highway", "footway")])).is_none());
    assert!(road_class(&tags(&[("highway", "service")])).is_some());
}

fn junction_control(signal_offset: f64, junction_tags: &[(&str, &str)]) -> Control {
    let mut fixture = Fixture::default();
    four_way(&mut fixture)
        .tagged_node(5, LAT, LON, junction_tags)
        .tagged_node(
            6,
            LAT,
            LON - signal_offset * STEP,
            &[("highway", "traffic_signals")],
        )
        .way(101, &[1, 6, 5], &[("highway", "residential")]);
    let junction_index = 4;
    fixture.build().nodes.control[junction_index]
}

#[test]
fn control_assignment() {
    assert_eq!(junction_control(1.0, &[]), Control::Signal);
    assert_eq!(junction_control(4.0, &[]), Control::Priority);
    assert_eq!(junction_control(4.0, &[("highway", "stop")]), Control::Stop);
    assert_eq!(
        junction_control(1.0, &[("highway", "stop")]),
        Control::Signal
    );
    let all_way = [("highway", "stop"), ("stop", "all")];
    assert_eq!(junction_control(4.0, &all_way), Control::AllWayStop);
}

fn ban_count(build: impl FnOnce(&mut Fixture)) -> usize {
    let mut fixture = Fixture::default();
    four_way(&mut fixture);
    build(&mut fixture);
    fixture.build().turn_bans.len()
}

#[test]
fn restriction_expansion() {
    assert_eq!(
        ban_count(|f| {
            f.relation(restriction(1, "no_left_turn", 101, 5, 202));
        }),
        1
    );
    assert_eq!(
        ban_count(|f| {
            f.relation(restriction(1, "only_straight_on", 101, 5, 102));
        }),
        2
    );
    let via_way = |f: &mut Fixture| {
        f.relation(restriction(1, "no_left_turn", 101, 5, 202));
        if let Some(relation) = f.osm.relations.get_mut(&1) {
            relation.members[1].kind = "way".to_string();
        }
    };
    assert_eq!(ban_count(via_way), 0);
    let through = |f: &mut Fixture| {
        f.way(101, &[1, 5, 2], &[("highway", "residential")]);
        f.osm.ways.remove(&102);
        f.relation(restriction(1, "no_left_turn", 101, 5, 202));
    };
    assert_eq!(ban_count(through), 0);
}

#[test]
fn closed_way_has_no_self_loop() {
    let mut fixture = Fixture::default();
    fixture
        .node(1, LAT, LON)
        .node(2, LAT, LON + STEP)
        .node(3, LAT + STEP, LON + STEP)
        .node(4, LAT + STEP, LON)
        .node(5, LAT - STEP, LON)
        .way(10, &[1, 2, 3, 4, 1], &[("highway", "residential")])
        .way(11, &[5, 1], &[("highway", "residential")]);
    let map = fixture.build();
    assert!((0..map.road_count()).all(|r| map.roads.from[r] != map.roads.to[r]));
    assert_eq!(validate(&map), Ok(()));
}

#[test]
fn projection() {
    let projection = Projection::centered(LAT - 0.01, LON - 0.01, LAT + 0.01, LON + 0.01);
    let (x, y) = projection.project(LAT, LON);
    assert!(x.abs() < 1e-6 && y.abs() < 1e-6);
    let (_, y) = projection.project(LAT + 0.001, LON);
    assert!((y + 111.19).abs() < 0.01, "y = {y}");
}

#[test]
fn lasso_way_has_no_self_loop() {
    let mut fixture = Fixture::default();
    fixture
        .node(1, LAT - STEP, LON)
        .node(2, LAT, LON)
        .node(3, LAT, LON + STEP)
        .node(4, LAT + STEP, LON + STEP)
        .way(10, &[1, 2, 3, 4, 2], &[("highway", "residential")]);
    let map = fixture.build();
    assert!((0..map.road_count()).all(|r| map.roads.from[r] != map.roads.to[r]));
    assert_eq!(validate(&map), Ok(()));
}

#[test]
fn unknown_element_types_are_skipped() {
    let json = r#"{ "elements": [
      { "type": "area", "id": 3600000001, "tags": { "name": "x" } },
      { "type": "count", "id": 0, "tags": { "total": "5" } },
      { "type": "node", "id": 1, "lat": 14.6, "lon": 121.0 }
    ] }"#;
    let mut osm = OsmData::default();
    let mut stats = LoadStats::default();
    match parse(json.as_bytes()) {
        Ok(response) => merge(&mut osm, &mut stats, response),
        Err(error) => panic!("parse failed: {error}"),
    }
    assert_eq!(osm.nodes.len(), 1);
}

#[test]
fn empty_name_is_unnamed() {
    let map = single_way_map(&[("highway", "residential"), ("name", "")]);
    assert_eq!(map.names, vec![String::new()]);
    assert_eq!(map.roads.name[0], 0);
}
