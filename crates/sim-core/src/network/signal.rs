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

#[derive(Clone, Debug, Default)]
pub struct Signals {
    clusters: Vec<SignalCluster>,
    node_cluster: BTreeMap<u32, u32>,
    approach_phase: BTreeMap<LinkId, (u32, usize)>,
    internal: Vec<LinkId>,
}

impl Signals {
    pub fn build(
        roads: &RoadStore,
        nodes: &NodeStore,
        spatial: &SpatialGrid,
        is_active: impl Fn(u32) -> bool,
    ) -> Signals {
        let groups = cluster::group(nodes, spatial, &is_active);
        let mut signals = Signals::default();
        for members in groups {
            signals.add_cluster(roads, nodes, &is_active, members);
        }
        signals.internal.sort_unstable();
        signals
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
                self.approach_phase.insert(link, (index, phase));
            }
        }
        for &node in &members {
            self.node_cluster.insert(node, index);
        }
        self.internal.extend(links.internal);
        self.clusters.push(SignalCluster {
            nodes: members,
            approaches: links.approaches,
            green_ticks: vec![SIGNAL_GREEN_TICKS; phases.len()],
            phases,
            offset_ticks: 0,
        });
    }

    pub fn state(&self, link: LinkId, tick: u64) -> Option<SignalState> {
        let &(cluster, phase) = self.approach_phase.get(&link)?;
        let cluster = &self.clusters[cluster as usize];
        Some(phase_state(cluster, phase, tick))
    }

    pub fn cluster_of(&self, node: u32) -> Option<u32> {
        self.node_cluster.get(&node).copied()
    }

    pub fn is_internal(&self, link: LinkId) -> bool {
        self.internal.binary_search(&link).is_ok()
    }

    pub fn clusters(&self) -> &[SignalCluster] {
        &self.clusters
    }
}

fn window(green: u32) -> u64 {
    u64::from(green + SIGNAL_AMBER_TICKS + SIGNAL_ALL_RED_TICKS)
}

fn phase_state(cluster: &SignalCluster, phase: usize, tick: u64) -> SignalState {
    let cycle: u64 = cluster.green_ticks.iter().map(|&g| window(g)).sum();
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
