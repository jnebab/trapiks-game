use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::consts::TICKS_PER_SECOND;
use crate::map::{Control, RoadClass};
use crate::network::{Network, SignalCluster, TurnKind, road_of};
use crate::sim::Sim;

use super::flyover_rules::build_flyover;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RoadInspection {
    pub road: u32,
    pub class: RoadClass,
    pub length_m: f64,
    pub lanes_forward: u8,
    pub lanes_backward: u8,
    pub speed_kph: u8,
    pub layer: i8,
    pub deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnInfo {
    pub from_road: u32,
    pub to_road: u32,
    pub kind: TurnKind,
    pub allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SignalInfo {
    pub phase_count: u32,
    pub greens_s: Vec<u32>,
    pub offset_s: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NodeInspection {
    pub node: u32,
    pub control: Control,
    pub roads: Vec<u32>,
    pub signal: Option<SignalInfo>,
    pub turns: Vec<TurnInfo>,
    pub flyover_pairs: Vec<[u32; 2]>,
}

impl Sim {
    pub fn inspect_road(&self, road: u32) -> Option<RoadInspection> {
        inspect_road(self.network(), road)
    }

    pub fn inspect_node(&self, node: u32) -> Option<NodeInspection> {
        inspect_node(self.network(), node)
    }
}

pub fn inspect_road(network: &Network, road: u32) -> Option<RoadInspection> {
    let roads = &network.roads;
    let index = road as usize;
    if index >= roads.count() {
        return None;
    }
    Some(RoadInspection {
        road,
        class: roads.class[index],
        length_m: roads.length[index],
        lanes_forward: roads.lanes_forward[index],
        lanes_backward: roads.lanes_backward[index],
        speed_kph: roads.speed_kph[index],
        layer: roads.layer[index],
        deleted: roads.deleted[index],
    })
}

pub fn inspect_node(network: &Network, node: u32) -> Option<NodeInspection> {
    if node as usize >= network.nodes.count() {
        return None;
    }
    let roads = live_roads(network, node);
    Some(NodeInspection {
        node,
        control: network.nodes.control[node as usize],
        signal: network.signals().cluster_at(node).map(signal_info),
        turns: turn_infos(network, node),
        flyover_pairs: flyover_pairs(network, node, &roads),
        roads,
    })
}

fn live_roads(network: &Network, node: u32) -> Vec<u32> {
    let mut roads: Vec<u32> = network.nodes.roads[node as usize]
        .iter()
        .copied()
        .filter(|&road| network.roads.is_live(road))
        .collect();
    roads.sort_unstable();
    roads.dedup();
    roads
}

fn signal_info(cluster: &SignalCluster) -> SignalInfo {
    SignalInfo {
        phase_count: cluster.phases.len() as u32,
        greens_s: cluster
            .green_ticks
            .iter()
            .map(|&ticks| ticks / TICKS_PER_SECOND)
            .collect(),
        offset_s: cluster.offset_ticks / TICKS_PER_SECOND,
    }
}

fn turn_infos(network: &Network, node: u32) -> Vec<TurnInfo> {
    let mut turns: Vec<TurnInfo> = network
        .turns_ignoring_bans(node)
        .into_iter()
        .map(|(from, to, kind)| {
            let (from_road, to_road) = (road_of(from), road_of(to));
            TurnInfo {
                from_road,
                to_road,
                kind,
                allowed: !network.is_banned(node, from_road, to_road),
            }
        })
        .collect();
    turns.sort_by_key(|turn| (turn.from_road, turn.to_road));
    turns
}

fn flyover_pairs(network: &Network, node: u32, roads: &[u32]) -> Vec<[u32; 2]> {
    let mut pairs = Vec::new();
    for (i, &a) in roads.iter().enumerate() {
        for &b in &roads[i + 1..] {
            if build_flyover(network, node, [a, b]).is_ok() {
                pairs.push([a, b]);
            }
        }
    }
    pairs
}
