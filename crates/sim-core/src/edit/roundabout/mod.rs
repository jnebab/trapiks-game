mod plan;
mod rules;

use crate::map::Control;
use crate::network::{Network, NewRoad};

use super::apply::{Edit, Scope};
use super::flyover::far_end;

pub use rules::{
    ROUNDABOUT_MAX_RADIUS, ROUNDABOUT_MIN_CHORD, ROUNDABOUT_MIN_RADIUS, build_roundabout,
};

const RING_SPEED_KPH: u8 = 30;

#[derive(Clone, Debug, PartialEq)]
pub struct ArmRecord {
    pub road: u32,
    pub old_range: (u32, u32),
    pub old_length: f64,
    pub arrived: bool,
    pub cut: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoundaboutUndo {
    pub node: u32,
    pub arms: Vec<ArmRecord>,
    pub road_len_before: u32,
    pub node_len_before: u32,
}

impl RoundaboutUndo {
    pub fn ring_node(&self, arm: usize) -> u32 {
        self.node_len_before + arm as u32
    }

    pub fn ring_nodes(&self) -> Vec<u32> {
        (0..self.arms.len())
            .map(|arm| self.ring_node(arm))
            .collect()
    }

    pub fn ring_roads(&self) -> Vec<u32> {
        let count = self.arms.len() as u32;
        (self.road_len_before..self.road_len_before + count).collect()
    }

    pub fn rebuild(&self) -> Edit {
        Edit::BuildRoundabout {
            node: self.node,
            radius_m: self.arms.first().map_or(0, |arm| arm.cut as u8),
        }
    }

    pub fn is_ring_link(&self, link: u32) -> bool {
        link >= self.road_len_before * 2
    }
}

pub fn build(network: &mut Network, node: u32, radius_m: u8) -> RoundaboutUndo {
    let road_len_before = network.roads.count() as u32;
    let node_len_before = network.nodes.count() as u32;
    let radius = f64::from(radius_m);
    let ring = plan::plan(network, node, radius);
    let arms = ring
        .arms
        .iter()
        .map(|arm| cut_arm(network, node, arm, radius))
        .collect();
    for ((i, j), arc) in ring.pairs().zip(&ring.arcs) {
        let row = NewRoad {
            ends: (node_len_before + i as u32, node_len_before + j as u32),
            class: ring.class,
            lanes: (ring.lanes, 0),
            layer: ring.layer,
            name: 0,
            speed_kph: RING_SPEED_KPH,
            roundabout: true,
        };
        network.append_new_road(&row, arc);
    }
    RoundaboutUndo {
        node,
        arms,
        road_len_before,
        node_len_before,
    }
}

fn cut_arm(network: &mut Network, node: u32, arm: &plan::ArmCut, radius: f64) -> ArmRecord {
    let ring_node = network.append_node(arm.point, Control::Yield);
    let (old_range, old_length) = network.repoint_road(arm.road, &arm.outer);
    network.set_endpoint(arm.road, node, ring_node);
    ArmRecord {
        road: arm.road,
        old_range,
        old_length,
        arrived: arm.arrived,
        cut: radius,
    }
}

pub fn restore_arms(network: &mut Network, undo: &RoundaboutUndo) {
    for (index, arm) in undo.arms.iter().enumerate() {
        network.restore_geometry(arm.road, arm.old_range, arm.old_length);
        network.set_endpoint(arm.road, undo.ring_node(index), undo.node);
    }
    for road in undo.ring_roads() {
        network.set_road_deleted(road, true);
    }
}

pub fn truncate(network: &mut Network, undo: &RoundaboutUndo) {
    network.truncate_roads(undo.road_len_before as usize);
    network.truncate_nodes(undo.node_len_before as usize);
}

pub fn undo(network: &mut Network, undo: &RoundaboutUndo) {
    restore_arms(network, undo);
    truncate(network, undo);
}

pub fn build_scope(network: &Network, node: u32) -> Scope {
    let arms = plan::arms(network, node);
    let roads_before = network.roads.count() as u32;
    let nodes_before = network.nodes.count() as u32;
    let count = arms.len() as u32;
    let far: Vec<u32> = arms
        .iter()
        .map(|&road| far_end(network, road, node))
        .collect();
    changed(
        node,
        &arms,
        (roads_before..roads_before + count).collect(),
        (nodes_before..nodes_before + count).collect(),
        &far,
    )
}

pub fn undo_scope(network: &Network, undo: &RoundaboutUndo) -> Scope {
    let roads: Vec<u32> = undo.arms.iter().map(|arm| arm.road).collect();
    let far: Vec<u32> = (0..undo.arms.len())
        .map(|i| far_end(network, undo.arms[i].road, undo.ring_node(i)))
        .collect();
    changed(
        undo.node,
        &roads,
        undo.ring_roads(),
        undo.ring_nodes(),
        &far,
    )
}

fn changed(node: u32, arms: &[u32], ring: Vec<u32>, added: Vec<u32>, far: &[u32]) -> Scope {
    let mut nodes = vec![node];
    nodes.extend(added);
    nodes.sort_unstable();
    let mut invalidated = nodes.clone();
    invalidated.extend_from_slice(far);
    invalidated.sort_unstable();
    invalidated.dedup();
    let mut roads = arms.to_vec();
    roads.extend(ring);
    Scope {
        roads,
        nodes,
        invalidated,
        signals: true,
    }
}
