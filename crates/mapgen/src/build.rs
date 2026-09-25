use std::collections::BTreeMap;

use trapiks_sim_core::map::{Control, MapData, NodeTable, PointTable, RoadTable, validate};

use crate::controls::{self, Positions};
use crate::filter::road_class;
use crate::input::{OsmData, OsmWay};
use crate::project::Projection;
use crate::restrictions;
use crate::tags::{self, Oneway};
use crate::topology::{self, Attributes, TopoRoad, Topology, WayInput};

pub fn build(osm: &OsmData) -> Result<MapData, String> {
    let ways: Vec<WayInput> = osm
        .ways
        .iter()
        .filter_map(|(id, way)| way_input(osm, *id, way))
        .collect();
    let topology = topology::build(&ways);
    let projection = projection_for(osm, &topology.roads)?;
    let positions = project_nodes(osm, &topology.roads, &projection);
    let node_index: BTreeMap<i64, u32> = topology.nodes.iter().copied().zip(0u32..).collect();
    let map = MapData {
        origin: projection.origin(),
        nodes: node_table(osm, &topology, &positions),
        turn_bans: restrictions::expand(osm, &topology.roads, &node_index),
        ..road_tables(&topology.roads, &node_index, &positions)
    };
    validate(&map).map_err(|e| e.to_string())?;
    Ok(map)
}

fn way_input(osm: &OsmData, id: i64, way: &OsmWay) -> Option<WayInput> {
    let class = road_class(&way.tags)?;
    let mut nodes = topology::clean_nodes(&way.nodes);
    if nodes.len() < 2 || nodes.iter().any(|node| !osm.nodes.contains_key(node)) {
        return None;
    }
    let oneway = tags::oneway(&way.tags, class);
    let (mut forward, mut backward) = tags::lanes(&way.tags, class, oneway);
    if oneway == Oneway::Backward {
        nodes.reverse();
        (forward, backward) = (backward, forward);
    }
    let attributes = Attributes {
        class,
        lanes_forward: forward,
        lanes_backward: backward,
        speed_kph: tags::speed_kph(&way.tags, class),
        layer: tags::layer(&way.tags),
        name: tags::name(&way.tags).map(str::to_string),
    };
    Some(WayInput {
        id,
        nodes,
        attributes,
    })
}

fn used_nodes(roads: &[TopoRoad]) -> impl Iterator<Item = i64> + '_ {
    roads.iter().flat_map(|road| road.node_ids.iter().copied())
}

fn projection_for(osm: &OsmData, roads: &[TopoRoad]) -> Result<Projection, String> {
    let mut coords = used_nodes(roads).filter_map(|id| osm.nodes.get(&id));
    let first = coords.next().ok_or("no drivable roads in input")?;
    let init = (first.lat, first.lon, first.lat, first.lon);
    let (min_lat, min_lon, max_lat, max_lon) = coords.fold(init, |b, n| {
        (
            b.0.min(n.lat),
            b.1.min(n.lon),
            b.2.max(n.lat),
            b.3.max(n.lon),
        )
    });
    Ok(Projection::centered(min_lat, min_lon, max_lat, max_lon))
}

fn project_nodes(osm: &OsmData, roads: &[TopoRoad], projection: &Projection) -> Positions {
    used_nodes(roads)
        .filter_map(|id| {
            osm.nodes
                .get(&id)
                .map(|n| (id, projection.project(n.lat, n.lon)))
        })
        .collect()
}

fn position(positions: &Positions, node: i64) -> (f32, f32) {
    let (x, y) = positions.get(&node).copied().unwrap_or_default();
    (x as f32, y as f32)
}

fn node_table(osm: &OsmData, topology: &Topology, positions: &Positions) -> NodeTable {
    let controls = controls::assign(osm, &topology.roads, positions);
    let mut table = NodeTable::default();
    for node in &topology.nodes {
        let (x, y) = position(positions, *node);
        table.x.push(x);
        table.y.push(y);
        table
            .control
            .push(controls.get(node).copied().unwrap_or(Control::Priority));
    }
    table
}

fn road_tables(
    roads: &[TopoRoad],
    node_index: &BTreeMap<i64, u32>,
    positions: &Positions,
) -> MapData {
    let mut map = MapData {
        names: vec![String::new()],
        ..MapData::default()
    };
    map.roads.point_start.push(0);
    let mut name_lookup = BTreeMap::new();
    for road in roads {
        let name = name_index(&mut map.names, &mut name_lookup, road);
        push_road(&mut map, road, name, node_index, positions);
    }
    map
}

fn push_road(
    map: &mut MapData,
    road: &TopoRoad,
    name: u32,
    node_index: &BTreeMap<i64, u32>,
    positions: &Positions,
) {
    let table: &mut RoadTable = &mut map.roads;
    let attributes = &road.attributes;
    table
        .from
        .push(node_index.get(&road.from_node()).copied().unwrap_or(0));
    table
        .to
        .push(node_index.get(&road.to_node()).copied().unwrap_or(0));
    table.class.push(attributes.class);
    table.lanes_forward.push(attributes.lanes_forward);
    table.lanes_backward.push(attributes.lanes_backward);
    table.speed_kph.push(attributes.speed_kph);
    table.layer.push(attributes.layer);
    table.name.push(name);
    push_points(&mut map.points, road, positions);
    let point_count = u32::try_from(map.points.x.len()).unwrap_or(u32::MAX);
    map.roads.point_start.push(point_count);
}

fn push_points(points: &mut PointTable, road: &TopoRoad, positions: &Positions) {
    for node in &road.node_ids {
        let (x, y) = position(positions, *node);
        points.x.push(x);
        points.y.push(y);
    }
}

fn name_index(names: &mut Vec<String>, lookup: &mut BTreeMap<String, u32>, road: &TopoRoad) -> u32 {
    let Some(name) = road.attributes.name.as_ref() else {
        return 0;
    };
    if let Some(&index) = lookup.get(name) {
        return index;
    }
    let index = u32::try_from(names.len()).unwrap_or(u32::MAX);
    names.push(name.clone());
    lookup.insert(name.clone(), index);
    index
}
