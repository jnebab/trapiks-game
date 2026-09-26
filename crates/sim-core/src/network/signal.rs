mod cluster;
mod phases;

use std::collections::BTreeMap;

use crate::consts::{SIGNAL_ALL_RED_TICKS, SIGNAL_AMBER_TICKS, SIGNAL_GREEN_TICKS};

use super::link::LinkId;
use super::node::NodeStore;
use super::road::RoadStore;
use super::spatial::SpatialGrid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalState {
    Green,
    Amber,
    Red,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignalCluster {
    pub nodes: Vec<u32>,
    pub approaches: Vec<LinkId>,
    pub phases: Vec<Vec<LinkId>>,
    pub green_ticks: Vec<u32>,
    pub offset_ticks: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timing {
    pub green_ticks: Vec<u32>,
    pub offset_ticks: u32,
}

impl SignalCluster {
    pub fn key(&self) -> u32 {
        self.nodes.first().copied().unwrap_or_default()
    }

    pub fn timing(&self) -> Timing {
        Timing {
            green_ticks: self.green_ticks.clone(),
            offset_ticks: self.offset_ticks,
        }
    }
}

pub struct SignalInputs<'a> {
    pub roads: &'a RoadStore,
    pub nodes: &'a NodeStore,
    pub spatial: &'a SpatialGrid,
    pub timings: &'a BTreeMap<u32, Timing>,
}

#[derive(Clone, Debug, Default)]
pub struct Signals {
    clusters: Vec<SignalCluster>,
    node_cluster: Vec<Option<u32>>,
    approach_phase: Vec<Option<(u32, u16)>>,
    internal: Vec<bool>,
}

impl Signals {
    pub fn build(inputs: &SignalInputs, is_active: impl Fn(u32) -> bool) -> Signals {
        let (roads, nodes) = (inputs.roads, inputs.nodes);
        let groups = cluster::group(nodes, inputs.spatial, &is_active);
        let mut signals = Signals {
            approach_phase: vec![None; roads.count() * 2],
            node_cluster: vec![None; nodes.count()],
            internal: vec![false; roads.count() * 2],
            ..Signals::default()
        };
        for members in groups {
            signals.add_cluster(roads, nodes, &is_active, members);
        }
        signals.apply_timings(inputs.timings);
        signals
    }

    fn apply_timings(&mut self, timings: &BTreeMap<u32, Timing>) {
        for cluster in &mut self.clusters {
            let Some(timing) = timings.get(&cluster.key()) else {
                continue;
            };
            if timing.green_ticks.len() == cluster.phases.len() {
                cluster.green_ticks.clone_from(&timing.green_ticks);
                cluster.offset_ticks = timing.offset_ticks;
            }
        }
    }

    fn add_cluster(
        &mut self,
        roads: &RoadStore,
        nodes: &NodeStore,
        is_active: &impl Fn(u32) -> bool,
        members: Vec<u32>,
    ) {
        let index = self.clusters.len() as u32;
        let links = cluster::classify_links(roads, nodes, is_active, &members);
        let phases = phases::plan(roads, &links.approaches);
        for (phase, group) in phases.iter().enumerate() {
            for &link in group {
                if let Some(entry) = self.approach_phase.get_mut(link as usize) {
                    *entry = Some((index, phase as u16));
                }
            }
        }
        for &node in &members {
            if let Some(entry) = self.node_cluster.get_mut(node as usize) {
                *entry = Some(index);
            }
        }
        for link in links.internal {
            if let Some(entry) = self.internal.get_mut(link as usize) {
                *entry = true;
            }
        }
        self.clusters.push(SignalCluster {
            nodes: members,
            approaches: links.approaches,
            green_ticks: vec![SIGNAL_GREEN_TICKS; phases.len()],
            phases,
            offset_ticks: 0,
        });
    }

    pub fn state(&self, link: LinkId, tick: u64) -> Option<SignalState> {
        let (cluster, phase) = (*self.approach_phase.get(link as usize)?)?;
        let cluster = &self.clusters[cluster as usize];
        Some(phase_state(cluster, usize::from(phase), tick))
    }

    pub fn cluster_of(&self, node: u32) -> Option<u32> {
        self.node_cluster.get(node as usize).copied().flatten()
    }

    pub fn is_internal(&self, link: LinkId) -> bool {
        self.internal.get(link as usize).copied().unwrap_or(false)
    }

    pub fn clusters(&self) -> &[SignalCluster] {
        &self.clusters
    }

    pub fn cluster_at(&self, node: u32) -> Option<&SignalCluster> {
        self.clusters.get(self.cluster_of(node)? as usize)
    }
}

pub fn cycle_ticks(green_ticks: &[u32]) -> u64 {
    green_ticks.iter().map(|&g| window(g)).sum()
}

fn window(green: u32) -> u64 {
    u64::from(green + SIGNAL_AMBER_TICKS + SIGNAL_ALL_RED_TICKS)
}

fn phase_state(cluster: &SignalCluster, phase: usize, tick: u64) -> SignalState {
    let cycle = cycle_ticks(&cluster.green_ticks);
    let t = (tick + u64::from(cluster.offset_ticks)) % cycle;
    let start: u64 = cluster.green_ticks[..phase]
        .iter()
        .map(|&g| window(g))
        .sum();
    let green_end = start + u64::from(cluster.green_ticks[phase]);
    let amber_end = green_end + u64::from(SIGNAL_AMBER_TICKS);
    if (start..green_end).contains(&t) {
        return SignalState::Green;
    }
    if (green_end..amber_end).contains(&t) {
        return SignalState::Amber;
    }
    SignalState::Red
}
