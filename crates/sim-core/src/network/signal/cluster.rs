use crate::consts::SIGNAL_CLUSTER_RADIUS;
use crate::map::Control;
use crate::network::link::{LinkId, arriving, from_node, lanes};
use crate::network::node::NodeStore;
use crate::network::road::RoadStore;
use crate::network::spatial::SpatialGrid;

pub struct ClusterLinks {
    pub approaches: Vec<LinkId>,
    pub internal: Vec<LinkId>,
}

pub fn group(
    nodes: &NodeStore,
    spatial: &SpatialGrid,
    is_active: &impl Fn(u32) -> bool,
) -> Vec<Vec<u32>> {
    let signalized: Vec<u32> = (0..nodes.count() as u32)
        .filter(|&n| is_signalized(nodes, is_active, n))
        .collect();
    let mut parent: Vec<usize> = (0..signalized.len()).collect();
    for (i, &node) in signalized.iter().enumerate() {
        for other in spatial.nodes_within(nodes, nodes.pos[node as usize], SIGNAL_CLUSTER_RADIUS) {
            if let Ok(j) = signalized.binary_search(&other) {
                union(&mut parent, i, j);
            }
        }
    }
    collect_groups(&signalized, &mut parent)
}

fn is_signalized(nodes: &NodeStore, is_active: &impl Fn(u32) -> bool, node: u32) -> bool {
    nodes.control[node as usize] == Control::Signal && nodes.active_degree(node, is_active) >= 3
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(parent, a), find(parent, b));
    parent[ra.max(rb)] = ra.min(rb);
}

fn collect_groups(signalized: &[u32], parent: &mut [usize]) -> Vec<Vec<u32>> {
    let mut group_of_root: Vec<Option<usize>> = vec![None; signalized.len()];
    let mut groups: Vec<Vec<u32>> = Vec::new();
    for (i, &node) in signalized.iter().enumerate() {
        let root = find(parent, i);
        let group = *group_of_root[root].get_or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[group].push(node);
    }
    groups
}

pub fn classify_links(
    roads: &RoadStore,
    nodes: &NodeStore,
    is_active: &impl Fn(u32) -> bool,
    members: &[u32],
) -> ClusterLinks {
    let mut links = ClusterLinks {
        approaches: Vec::new(),
        internal: Vec::new(),
    };
    for &node in members {
        for &road in &nodes.roads[node as usize] {
            if is_active(road) {
                add_arriving(roads, members, &mut links, arriving(roads, road, node));
            }
        }
    }
    links.approaches.sort_unstable();
    links.internal.sort_unstable();
    links.internal.dedup();
    links
}

fn add_arriving(roads: &RoadStore, members: &[u32], links: &mut ClusterLinks, link: LinkId) {
    if lanes(roads, link) == 0 {
        return;
    }
    if members.binary_search(&from_node(roads, link)).is_ok() {
        links.internal.push(link);
    } else {
        links.approaches.push(link);
    }
}
