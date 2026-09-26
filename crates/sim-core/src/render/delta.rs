use crate::network::Network;

use super::approach_markers::{ApproachMarkers, markers_at};
use super::junction_shape::{JunctionShapes, node_shapes};
use super::setbacks::road_end_setbacks;
use super::signal_pills::{SignalPills, signal_pills};

pub type JunctionRows = JunctionShapes;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoadRows {
    pub id: Vec<u32>,
    pub class: Vec<u8>,
    pub lanes_forward: Vec<u8>,
    pub lanes_backward: Vec<u8>,
    pub layer: Vec<i8>,
    pub name: Vec<u32>,
    pub deleted: Vec<u8>,
    pub from: Vec<u32>,
    pub to: Vec<u32>,
    pub point_len: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeRows {
    pub id: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub control: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SetbackRows {
    pub id: Vec<u32>,
    pub start: Vec<f32>,
    pub end: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkerRows {
    pub markers: ApproachMarkers,
    pub nodes: Vec<u32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetworkDelta {
    pub road_count: u32,
    pub node_count: u32,
    pub roads: RoadRows,
    pub nodes: NodeRows,
    pub setbacks: SetbackRows,
    pub junctions: JunctionRows,
    pub markers: MarkerRows,
    pub pills: SignalPills,
}

pub fn network_delta(network: &Network, roads: &[u32], nodes: &[u32]) -> NetworkDelta {
    let roads = live_ids(roads, network.roads.count());
    let nodes = affected_nodes(network, &roads, nodes);
    NetworkDelta {
        road_count: network.roads.count() as u32,
        node_count: network.nodes.count() as u32,
        roads: road_rows(network, &roads),
        nodes: node_rows(network, &nodes),
        setbacks: setback_rows(network, &incident_roads(network, &nodes)),
        junctions: node_shapes(network, &nodes),
        markers: MarkerRows {
            markers: markers_at(network, &nodes),
            nodes: nodes.clone(),
        },
        pills: signal_pills(network),
    }
}

fn live_ids(ids: &[u32], count: usize) -> Vec<u32> {
    let mut live: Vec<u32> = ids
        .iter()
        .copied()
        .filter(|&id| (id as usize) < count)
        .collect();
    live.sort_unstable();
    live.dedup();
    live
}

fn affected_nodes(network: &Network, roads: &[u32], nodes: &[u32]) -> Vec<u32> {
    let endpoints = roads.iter().flat_map(|&road| {
        let index = road as usize;
        [network.roads.from[index], network.roads.to[index]]
    });
    let all: Vec<u32> = nodes.iter().copied().chain(endpoints).collect();
    live_ids(&all, network.nodes.count())
}

fn incident_roads(network: &Network, nodes: &[u32]) -> Vec<u32> {
    let all: Vec<u32> = nodes
        .iter()
        .flat_map(|&node| network.nodes.roads[node as usize].iter().copied())
        .collect();
    live_ids(&all, network.roads.count())
}

fn road_rows(network: &Network, roads: &[u32]) -> RoadRows {
    let mut rows = RoadRows::default();
    for &road in roads {
        push_road(network, road, &mut rows);
    }
    rows
}

fn push_road(network: &Network, road: u32, rows: &mut RoadRows) {
    let store = &network.roads;
    let index = road as usize;
    let points = store.points(road);
    rows.id.push(road);
    rows.class.push(store.class[index].code());
    rows.lanes_forward.push(store.lanes_forward[index]);
    rows.lanes_backward.push(store.lanes_backward[index]);
    rows.layer.push(store.layer[index]);
    rows.name.push(store.name[index]);
    rows.deleted.push(u8::from(store.deleted[index]));
    rows.from.push(store.from[index]);
    rows.to.push(store.to[index]);
    rows.point_len.push(points.len() as u32);
    rows.x.extend(points.iter().map(|p| p.x as f32));
    rows.y.extend(points.iter().map(|p| p.y as f32));
}

fn node_rows(network: &Network, nodes: &[u32]) -> NodeRows {
    let mut rows = NodeRows::default();
    for &node in nodes {
        let index = node as usize;
        let pos = network.nodes.pos[index];
        rows.id.push(node);
        rows.x.push(pos.x as f32);
        rows.y.push(pos.y as f32);
        rows.control.push(network.nodes.control[index].code());
    }
    rows
}

fn setback_rows(network: &Network, roads: &[u32]) -> SetbackRows {
    let mut rows = SetbackRows::default();
    for &road in roads {
        let [start, end] = road_end_setbacks(network, road);
        rows.id.push(road);
        rows.start.push(start);
        rows.end.push(end);
    }
    rows
}
