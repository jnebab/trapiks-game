mod merge;

use std::collections::{BTreeMap, BTreeSet};

use trapiks_sim_core::map::RoadClass;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attributes {
    pub class: RoadClass,
    pub lanes_forward: u8,
    pub lanes_backward: u8,
    pub speed_kph: u8,
    pub layer: i8,
    pub name: Option<String>,
}

impl Attributes {
    pub fn reversed(&self) -> Self {
        Self {
            lanes_forward: self.lanes_backward,
            lanes_backward: self.lanes_forward,
            ..self.clone()
        }
    }

    pub fn equals_reversed(&self, other: &Self) -> bool {
        self.lanes_forward == other.lanes_backward
            && self.lanes_backward == other.lanes_forward
            && self.class == other.class
            && self.speed_kph == other.speed_kph
            && self.layer == other.layer
            && self.name == other.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WayInput {
    pub id: i64,
    pub nodes: Vec<i64>,
    pub attributes: Attributes,
}

pub type RoadKey = (i64, usize);

#[derive(Clone, Debug, PartialEq)]
pub struct TopoRoad {
    pub key: RoadKey,
    pub way_ids: Vec<i64>,
    pub node_ids: Vec<i64>,
    pub attributes: Attributes,
}

impl TopoRoad {
    pub fn from_node(&self) -> i64 {
        self.node_ids[0]
    }

    pub fn to_node(&self) -> i64 {
        self.node_ids[self.node_ids.len() - 1]
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Topology {
    pub nodes: Vec<i64>,
    pub roads: Vec<TopoRoad>,
}

pub fn clean_nodes(nodes: &[i64]) -> Vec<i64> {
    let mut cleaned = nodes.to_vec();
    cleaned.dedup();
    cleaned
}

pub fn build(ways: &[WayInput]) -> Topology {
    let junctions = junction_nodes(ways);
    let segments = ways
        .iter()
        .flat_map(|way| split_way(way, &junctions))
        .collect();
    let roads = merge::merge_chains(segments);
    let nodes = endpoint_nodes(&roads);
    Topology { nodes, roads }
}

fn junction_nodes(ways: &[WayInput]) -> BTreeSet<i64> {
    let mut way_counts: BTreeMap<i64, usize> = BTreeMap::new();
    let mut junctions = BTreeSet::new();
    for way in ways {
        add_way_junctions(way, &mut junctions);
        let unique: BTreeSet<i64> = way.nodes.iter().copied().collect();
        for node in unique {
            *way_counts.entry(node).or_default() += 1;
        }
    }
    let shared = way_counts.into_iter().filter(|&(_, count)| count >= 2);
    junctions.extend(shared.map(|(node, _)| node));
    junctions
}

fn add_way_junctions(way: &WayInput, junctions: &mut BTreeSet<i64>) {
    let nodes = &way.nodes;
    let (first, last) = (nodes[0], nodes[nodes.len() - 1]);
    junctions.insert(first);
    junctions.insert(last);
    if first == last {
        junctions.insert(nodes[nodes.len() / 2]);
    }
    let mut seen = BTreeSet::new();
    let repeated = nodes.iter().filter(|&&node| !seen.insert(node));
    junctions.extend(repeated.copied());
}

fn split_way(way: &WayInput, junctions: &BTreeSet<i64>) -> Vec<TopoRoad> {
    let mut segments = Vec::new();
    let mut start = 0;
    for index in 1..way.nodes.len() {
        if junctions.contains(&way.nodes[index]) {
            push_segment(way, start, index, &mut segments);
            start = index;
        }
    }
    segments
}

fn push_segment(way: &WayInput, start: usize, end: usize, segments: &mut Vec<TopoRoad>) {
    if way.nodes[start] == way.nodes[end] {
        let middle = start + (end - start) / 2;
        push_segment(way, start, middle, segments);
        push_segment(way, middle, end, segments);
        return;
    }
    segments.push(TopoRoad {
        key: (way.id, start),
        way_ids: vec![way.id],
        node_ids: way.nodes[start..=end].to_vec(),
        attributes: way.attributes.clone(),
    });
}

fn endpoint_nodes(roads: &[TopoRoad]) -> Vec<i64> {
    let endpoints: BTreeSet<i64> = roads
        .iter()
        .flat_map(|road| [road.from_node(), road.to_node()])
        .collect();
    endpoints.into_iter().collect()
}
