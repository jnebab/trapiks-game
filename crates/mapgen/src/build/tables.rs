use std::collections::{BTreeMap, BTreeSet};

use trapiks_sim_core::map::{Control, MapData, NodeTable, PointTable, RoadTable};

use crate::controls::{self, Positions};
use crate::input::OsmData;
use crate::topology::{TopoRoad, Topology};

fn position(positions: &Positions, node: i64) -> (f32, f32) {
    let (x, y) = positions.get(&node).copied().unwrap_or_default();
    (x as f32, y as f32)
}

pub fn node_table(osm: &OsmData, topology: &Topology, positions: &Positions) -> NodeTable {
    let controls = controls::assign(osm, &topology.roads, positions);
    let entries = controls::roundabout_entries(&topology.roads);
    let mut table = NodeTable::default();
    for node in &topology.nodes {
        let (x, y) = position(positions, *node);
        table.x.push(x);
        table.y.push(y);
        table
            .control
            .push(default_control(&controls, &entries, *node));
    }
    table
}

fn default_control(
    controls: &BTreeMap<i64, Control>,
    entries: &BTreeSet<i64>,
    node: i64,
) -> Control {
    if let Some(&control) = controls.get(&node) {
        return control;
    }
    if entries.contains(&node) {
        return Control::Yield;
    }
    Control::Priority
}

pub fn road_tables(
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
    table.roundabout.push(attributes.roundabout);
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
