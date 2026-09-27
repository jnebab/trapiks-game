use super::{Direction, LinkId, Network, link_id};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub nodes: Vec<u32>,
    pub links: Vec<LinkId>,
}

impl Network {
    fn pending_version(&self) -> u64 {
        self.version + 1
    }

    pub(crate) fn touch_node(&mut self, node: u32) {
        let pending = self.pending_version();
        if let Some(stamp) = self.node_versions.get_mut(node as usize) {
            *stamp = pending;
        }
    }

    pub(crate) fn touch_road(&mut self, road: u32) {
        let pending = self.pending_version();
        if let Some(stamp) = self.road_versions.get_mut(road as usize) {
            *stamp = pending;
        }
    }

    pub(crate) fn mark_global(&mut self) {
        self.global_version = self.pending_version();
        let pending = self.global_version;
        self.node_versions.resize(self.nodes.count(), pending);
        self.road_versions.resize(self.roads.count(), pending);
    }

    pub fn global_version(&self) -> u64 {
        self.global_version
    }

    pub fn node_version(&self, node: u32) -> u64 {
        self.node_versions
            .get(node as usize)
            .copied()
            .unwrap_or(u64::MAX)
    }

    pub fn changes_since(&self, version: u64) -> Option<Changes> {
        if self.global_version > version {
            return None;
        }
        let nodes = stamped_after(&self.node_versions, version);
        let mut roads = stamped_after(&self.road_versions, version);
        for &node in &nodes {
            roads.extend_from_slice(&self.nodes.roads[node as usize]);
        }
        roads.sort_unstable();
        roads.dedup();
        let links = roads
            .iter()
            .flat_map(|&road| [Direction::Forward, Direction::Backward].map(|d| link_id(road, d)))
            .collect();
        Some(Changes { nodes, links })
    }
}

fn stamped_after(stamps: &[u64], version: u64) -> Vec<u32> {
    stamps
        .iter()
        .enumerate()
        .filter(|&(_, &stamp)| stamp > version)
        .map(|(index, _)| index as u32)
        .collect()
}
