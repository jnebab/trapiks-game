use crate::geom::split_polyline;
use crate::map::Control;
use crate::network::Network;

use super::apply::Scope;

const FLYOVER_REACH: f64 = 80.0;
const LAYER_NEIGHBOURHOOD: f64 = 1.0;

#[derive(Clone, Debug, PartialEq)]
pub struct FlyoverUndo {
    pub node: u32,
    pub roads: [u32; 2],
    pub old_ranges: [(u32, u32); 2],
    pub old_lengths: [f64; 2],
    pub old_ends: [u32; 2],
    pub road_len_before: u32,
    pub node_len_before: u32,
}

impl FlyoverUndo {
    pub fn top(&self) -> u32 {
        self.node_len_before
    }

    pub fn inner(&self, index: usize) -> u32 {
        self.road_len_before + index as u32
    }

    pub fn appended_nodes(&self) -> [u32; 3] {
        [self.top(), self.old_ends[0], self.old_ends[1]]
    }

    pub fn is_appended_link(&self, link: u32) -> bool {
        link >= self.road_len_before * 2
    }
}

pub fn reach(length: f64) -> f64 {
    FLYOVER_REACH.min(length / 2.0)
}

pub fn build(network: &mut Network, node: u32, through: [u32; 2]) -> FlyoverUndo {
    let road_len_before = network.roads.count() as u32;
    let node_len_before = network.nodes.count() as u32;
    let layer = flyover_layer(network, node, through);
    let top = network.append_node(network.nodes.pos[node as usize], Control::Priority);
    let lifted = through.map(|road| lift(network, road, node, (top, layer)));
    FlyoverUndo {
        node,
        roads: through,
        old_ranges: lifted.map(|l| l.range),
        old_lengths: lifted.map(|l| l.length),
        old_ends: lifted.map(|l| l.middle),
        road_len_before,
        node_len_before,
    }
}

#[derive(Clone, Copy)]
struct Lifted {
    range: (u32, u32),
    length: f64,
    middle: u32,
}

fn lift(network: &mut Network, road: u32, node: u32, (top, layer): (u32, i8)) -> Lifted {
    let index = road as usize;
    let arrives = network.roads.to[index] == node;
    let length = network.roads.length[index];
    let cut_s = if arrives {
        length - reach(length)
    } else {
        reach(length)
    };
    let (head, tail) = split_polyline(
        network.roads.points(road),
        network.roads.cumulative(road),
        cut_s,
    );
    let (outer, inner) = if arrives { (head, tail) } else { (tail, head) };
    let cut = if arrives { inner.first() } else { inner.last() };
    let middle = network.append_node(cut.copied().unwrap_or_default(), Control::Priority);
    let (old_range, old_length) = network.repoint_road(road, &outer);
    network.set_endpoint(road, node, middle);
    let ends = if arrives {
        (middle, top)
    } else {
        (top, middle)
    };
    network.append_road(road, ends, &inner, layer);
    Lifted {
        range: old_range,
        length: old_length,
        middle,
    }
}

fn flyover_layer(network: &Network, node: u32, through: [u32; 2]) -> i8 {
    let centre = network.nodes.pos[node as usize];
    let near = network
        .spatial
        .nodes_within(&network.nodes, centre, LAYER_NEIGHBOURHOOD);
    let incident = near
        .iter()
        .flat_map(|&n| network.nodes.roads[n as usize].iter().copied());
    let highest = incident
        .chain(through)
        .map(|road| network.roads.layer[road as usize])
        .max()
        .unwrap_or_default();
    highest.saturating_add(1)
}

pub fn restore_through(network: &mut Network, undo: &FlyoverUndo) {
    for index in 0..2 {
        let road = undo.roads[index];
        network.restore_geometry(road, undo.old_ranges[index], undo.old_lengths[index]);
        network.set_endpoint(road, undo.old_ends[index], undo.node);
        network.set_road_deleted(undo.inner(index), true);
    }
}

pub fn truncate(network: &mut Network, undo: &FlyoverUndo) {
    network.truncate_roads(undo.road_len_before as usize);
    network.truncate_nodes(undo.node_len_before as usize);
}

pub fn undo(network: &mut Network, undo: &FlyoverUndo) {
    restore_through(network, undo);
    truncate(network, undo);
}

pub fn build_scope(network: &Network, node: u32, through: [u32; 2]) -> Scope {
    let roads_before = network.roads.count() as u32;
    let nodes_before = network.nodes.count() as u32;
    let far = through.map(|road| far_end(network, road, node));
    changed(
        node,
        through,
        [roads_before, roads_before + 1],
        [nodes_before, nodes_before + 1, nodes_before + 2],
        far,
    )
}

pub fn undo_scope(network: &Network, undo: &FlyoverUndo) -> Scope {
    let far = [0, 1].map(|i| far_end(network, undo.roads[i], undo.old_ends[i]));
    changed(
        undo.node,
        undo.roads,
        [undo.inner(0), undo.inner(1)],
        undo.appended_nodes(),
        far,
    )
}

fn changed(node: u32, roads: [u32; 2], inner: [u32; 2], added: [u32; 3], far: [u32; 2]) -> Scope {
    let mut nodes = vec![node];
    nodes.extend(added);
    nodes.sort_unstable();
    let mut invalidated = nodes.clone();
    invalidated.extend(far);
    invalidated.sort_unstable();
    invalidated.dedup();
    let mut roads = roads.to_vec();
    roads.extend(inner);
    Scope {
        roads,
        nodes,
        invalidated,
        signals: true,
    }
}

pub(super) fn far_end(network: &Network, road: u32, near: u32) -> u32 {
    let index = road as usize;
    let from = network.roads.from[index];
    if from == near {
        network.roads.to[index]
    } else {
        from
    }
}
