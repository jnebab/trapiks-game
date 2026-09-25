use std::collections::BTreeMap;

use super::{RoadKey, TopoRoad};

struct Graph {
    roads: BTreeMap<RoadKey, TopoRoad>,
    ends: BTreeMap<i64, Vec<RoadKey>>,
}

pub fn merge_chains(segments: Vec<TopoRoad>) -> Vec<TopoRoad> {
    let mut graph = Graph::new(segments);
    while graph.merge_pass() {}
    graph.roads.into_values().collect()
}

impl Graph {
    fn new(segments: Vec<TopoRoad>) -> Self {
        let mut ends: BTreeMap<i64, Vec<RoadKey>> = BTreeMap::new();
        for road in &segments {
            ends.entry(road.from_node()).or_default().push(road.key);
            ends.entry(road.to_node()).or_default().push(road.key);
        }
        let roads = segments.into_iter().map(|road| (road.key, road)).collect();
        Self { roads, ends }
    }

    fn merge_pass(&mut self) -> bool {
        let keys: Vec<RoadKey> = self.roads.keys().copied().collect();
        let mut changed = false;
        for key in keys {
            while let Some(merged) = self.try_merge(key) {
                changed = true;
                if merged != key {
                    break;
                }
            }
        }
        changed
    }

    fn try_merge(&mut self, key: RoadKey) -> Option<RoadKey> {
        let road = self.roads.get(&key)?;
        let candidates = [road.to_node(), road.from_node()];
        candidates
            .into_iter()
            .find_map(|node| self.merge_at(key, node))
    }

    fn merge_at(&mut self, key: RoadKey, node: i64) -> Option<RoadKey> {
        let other_key = self.partner(key, node)?;
        let road = self.roads.get(&key)?;
        let other = self.roads.get(&other_key)?;
        let joined = join(road, other, node)?;
        self.replace(key, other_key, node, joined)
    }

    fn partner(&self, key: RoadKey, node: i64) -> Option<RoadKey> {
        match self.ends.get(&node)?.as_slice() {
            [a, b] if *a == key => Some(*b),
            [a, b] if *b == key => Some(*a),
            _ => None,
        }
    }

    fn replace(
        &mut self,
        key: RoadKey,
        other_key: RoadKey,
        node: i64,
        joined: TopoRoad,
    ) -> Option<RoadKey> {
        self.roads.remove(&key)?;
        self.roads.remove(&other_key)?;
        self.ends.remove(&node);
        for end in [joined.from_node(), joined.to_node()] {
            let keys = self.ends.entry(end).or_default();
            keys.retain(|k| *k != key && *k != other_key);
            keys.push(joined.key);
        }
        let merged = joined.key;
        self.roads.insert(merged, joined);
        Some(merged)
    }
}

fn join(road: &TopoRoad, other: &TopoRoad, node: i64) -> Option<TopoRoad> {
    let appending = road.to_node() == node;
    let other_near_is_from = other.from_node() == node;
    let flip = appending != other_near_is_from;
    let other_far = if other_near_is_from {
        other.to_node()
    } else {
        other.from_node()
    };
    let road_far = if appending {
        road.from_node()
    } else {
        road.to_node()
    };
    if other_far == road_far || !compatible(road, other, flip) {
        return None;
    }
    let other = oriented(other, flip);
    if appending {
        return Some(concat(road, &other));
    }
    Some(concat(&other, road))
}

fn compatible(road: &TopoRoad, other: &TopoRoad, flip: bool) -> bool {
    if flip {
        return road.attributes.equals_reversed(&other.attributes);
    }
    road.attributes == other.attributes
}

fn oriented(road: &TopoRoad, flip: bool) -> TopoRoad {
    if !flip {
        return road.clone();
    }
    let mut node_ids = road.node_ids.clone();
    node_ids.reverse();
    TopoRoad {
        node_ids,
        attributes: road.attributes.reversed(),
        ..road.clone()
    }
}

fn concat(first: &TopoRoad, second: &TopoRoad) -> TopoRoad {
    let mut node_ids = first.node_ids.clone();
    node_ids.extend_from_slice(&second.node_ids[1..]);
    let mut way_ids = [first.way_ids.clone(), second.way_ids.clone()].concat();
    way_ids.sort_unstable();
    way_ids.dedup();
    TopoRoad {
        key: first.key.min(second.key),
        way_ids,
        node_ids,
        attributes: first.attributes.clone(),
    }
}
