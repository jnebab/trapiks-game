mod geometry;
mod rules;

use crate::map::Control;
use crate::network::{Network, NewRoad};

use super::Endpoint;
use super::apply::Scope;

pub use geometry::{ADD_ROAD_SEGMENTS, AddRoadSpec, curve_points};
pub use rules::{ADD_ROAD_MIN_ANGLE, add_road};

use geometry::{class_and_speed, cut_road, plan_points};

#[derive(Clone, Debug, PartialEq)]
pub struct SplitRecord {
    pub road: u32,
    pub r2: u32,
    pub old_range: (u32, u32),
    pub old_length: f64,
    pub old_to: u32,
    pub cut: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AddRoadUndo {
    pub spec: AddRoadSpec,
    pub road_len_before: u32,
    pub node_len_before: u32,
    pub splits: Vec<SplitRecord>,
}

impl AddRoadUndo {
    pub fn new_road(&self) -> u32 {
        self.road_len_before + self.splits.len() as u32
    }

    pub fn split_node(&self, index: usize) -> u32 {
        self.node_len_before + index as u32
    }

    pub fn split_nodes(&self) -> Vec<u32> {
        (0..self.splits.len()).map(|i| self.split_node(i)).collect()
    }

    pub fn end_nodes(&self) -> [u32; 2] {
        resolved_nodes(&self.spec, self.node_len_before)
    }
}

fn resolved_nodes(spec: &AddRoadSpec, node_len_before: u32) -> [u32; 2] {
    let mut next = node_len_before;
    spec.ends().map(|end| match end {
        Endpoint::Node { node } => node,
        Endpoint::OnRoad { .. } => {
            next += 1;
            next - 1
        }
    })
}

pub fn build(network: &mut Network, spec: &AddRoadSpec) -> AddRoadUndo {
    let road_len_before = network.roads.count() as u32;
    let node_len_before = network.nodes.count() as u32;
    let points = plan_points(network, spec);
    let splits: Vec<SplitRecord> = spec
        .ends()
        .iter()
        .filter_map(|&end| match end {
            Endpoint::OnRoad { road, at_m } => Some(split(network, road, at_m)),
            Endpoint::Node { .. } => None,
        })
        .collect();
    let ends = resolved_nodes(spec, node_len_before);
    let (class, speed_kph) = class_and_speed(spec.lanes);
    let row = NewRoad {
        ends: (ends[0], ends[1]),
        class,
        lanes: spec.lanes,
        layer: spec.layer,
        name: 0,
        speed_kph,
        roundabout: false,
    };
    network.append_new_road(&row, &points);
    AddRoadUndo {
        spec: *spec,
        road_len_before,
        node_len_before,
        splits,
    }
}

fn split(network: &mut Network, road: u32, at_m: f64) -> SplitRecord {
    let (head, tail) = cut_road(network, road, at_m);
    let cut = head.last().copied().unwrap_or_default();
    let m = network.append_node(cut, Control::Priority);
    let old_to = network.roads.to[road as usize];
    let layer = network.roads.layer[road as usize];
    let (old_range, old_length) = network.repoint_road(road, &head);
    network.set_endpoint(road, old_to, m);
    let r2 = network.append_road(road, (m, old_to), &tail, layer);
    SplitRecord {
        road,
        r2,
        old_range,
        old_length,
        old_to,
        cut: at_m,
    }
}

pub fn restore_split(network: &mut Network, undo: &AddRoadUndo, index: usize) {
    let split = &undo.splits[index];
    network.restore_geometry(split.road, split.old_range, split.old_length);
    network.set_endpoint(split.road, undo.split_node(index), split.old_to);
}

pub fn delete_appended(network: &mut Network, undo: &AddRoadUndo) {
    network.set_road_deleted(undo.new_road(), true);
    for split in &undo.splits {
        network.set_road_deleted(split.r2, true);
    }
}

pub fn truncate(network: &mut Network, undo: &AddRoadUndo) {
    network.truncate_roads(undo.road_len_before as usize);
    network.truncate_nodes(undo.node_len_before as usize);
}

pub fn undo(network: &mut Network, undo: &AddRoadUndo) {
    for index in (0..undo.splits.len()).rev() {
        restore_split(network, undo, index);
    }
    delete_appended(network, undo);
    truncate(network, undo);
}

struct SplitIds {
    road: u32,
    r2: u32,
    m: u32,
    old_to: u32,
    start: u32,
}

pub fn build_scope(network: &Network, spec: &AddRoadSpec) -> Scope {
    let road_len_before = network.roads.count() as u32;
    let node_len_before = network.nodes.count() as u32;
    let splits: Vec<SplitIds> = spec
        .ends()
        .iter()
        .filter_map(|&end| match end {
            Endpoint::OnRoad { road, .. } => Some(road),
            Endpoint::Node { .. } => None,
        })
        .enumerate()
        .map(|(k, road)| SplitIds {
            road,
            r2: road_len_before + k as u32,
            m: node_len_before + k as u32,
            old_to: network.roads.to[road as usize],
            start: network.roads.from[road as usize],
        })
        .collect();
    let new_road = road_len_before + splits.len() as u32;
    changed(new_road, resolved_nodes(spec, node_len_before), &splits)
}

pub fn undo_scope(network: &Network, undo: &AddRoadUndo) -> Scope {
    let splits: Vec<SplitIds> = undo
        .splits
        .iter()
        .enumerate()
        .map(|(k, split)| SplitIds {
            road: split.road,
            r2: split.r2,
            m: undo.split_node(k),
            old_to: split.old_to,
            start: network.roads.from[split.road as usize],
        })
        .collect();
    changed(undo.new_road(), undo.end_nodes(), &splits)
}

fn changed(new_road: u32, ends: [u32; 2], splits: &[SplitIds]) -> Scope {
    let mut roads = vec![new_road];
    let mut nodes = ends.to_vec();
    let mut invalidated = Vec::new();
    for split in splits {
        roads.extend([split.road, split.r2]);
        nodes.extend([split.m, split.old_to]);
        invalidated.push(split.start);
    }
    roads.sort_unstable();
    roads.dedup();
    nodes.sort_unstable();
    nodes.dedup();
    invalidated.extend_from_slice(&nodes);
    invalidated.sort_unstable();
    invalidated.dedup();
    Scope {
        roads,
        nodes,
        invalidated,
        signals: true,
    }
}
