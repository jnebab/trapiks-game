use std::collections::BTreeMap;

use trapiks_sim_core::map::TurnBan;

use crate::input::{OsmData, OsmRelation};
use crate::tags::tag;
use crate::topology::TopoRoad;

struct RoadIndex<'a> {
    roads: &'a [TopoRoad],
    node_index: &'a BTreeMap<i64, u32>,
    by_way: BTreeMap<i64, Vec<u32>>,
    by_node: BTreeMap<i64, Vec<u32>>,
}

struct Restriction {
    only: bool,
    from_way: i64,
    via_node: i64,
    to_way: i64,
}

pub fn expand(osm: &OsmData, roads: &[TopoRoad], node_index: &BTreeMap<i64, u32>) -> Vec<TurnBan> {
    let index = RoadIndex::new(roads, node_index);
    let mut bans: Vec<TurnBan> = osm
        .relations
        .values()
        .filter_map(parse_restriction)
        .flat_map(|restriction| index.bans(&restriction))
        .collect();
    bans.sort_unstable();
    bans.dedup();
    bans
}

fn parse_restriction(relation: &OsmRelation) -> Option<Restriction> {
    if tag(&relation.tags, "type") != Some("restriction") {
        return None;
    }
    let value = tag(&relation.tags, "restriction")
        .or_else(|| tag(&relation.tags, "restriction:motorcar"))?;
    let only = value.starts_with("only_");
    if !only && !value.starts_with("no_") {
        return None;
    }
    Some(Restriction {
        only,
        from_way: single_member(relation, "from", "way")?,
        via_node: single_member(relation, "via", "node")?,
        to_way: single_member(relation, "to", "way")?,
    })
}

fn single_member(relation: &OsmRelation, role: &str, kind: &str) -> Option<i64> {
    let mut members = relation.members.iter().filter(|m| m.role == role);
    let member = members.next()?;
    if members.next().is_some() || member.kind != kind {
        return None;
    }
    Some(member.id)
}

impl<'a> RoadIndex<'a> {
    fn new(roads: &'a [TopoRoad], node_index: &'a BTreeMap<i64, u32>) -> Self {
        let mut by_way: BTreeMap<i64, Vec<u32>> = BTreeMap::new();
        let mut by_node: BTreeMap<i64, Vec<u32>> = BTreeMap::new();
        for (road, topo) in (0u32..).zip(roads) {
            for way in &topo.way_ids {
                by_way.entry(*way).or_default().push(road);
            }
            by_node.entry(topo.from_node()).or_default().push(road);
            by_node.entry(topo.to_node()).or_default().push(road);
        }
        Self {
            roads,
            node_index,
            by_way,
            by_node,
        }
    }

    fn bans(&self, restriction: &Restriction) -> Vec<TurnBan> {
        let via = restriction.via_node;
        let (Some(from_road), Some(to_road), Some(&via_node)) = (
            self.road_at(restriction.from_way, via),
            self.road_at(restriction.to_way, via),
            self.node_index.get(&via),
        ) else {
            return Vec::new();
        };
        let ban = |to: u32| TurnBan {
            via_node,
            from_road,
            to_road: to,
        };
        if !restriction.only {
            return vec![ban(to_road)];
        }
        self.incident(via)
            .filter(|&road| road != from_road && road != to_road)
            .map(ban)
            .collect()
    }

    fn road_at(&self, way: i64, node: i64) -> Option<u32> {
        let candidates = self.by_way.get(&way)?;
        let mut touching = candidates
            .iter()
            .copied()
            .filter(|&road| self.touches(road, node));
        let road = touching.next()?;
        if touching.next().is_some() {
            return None;
        }
        Some(road)
    }

    fn incident(&self, node: i64) -> impl Iterator<Item = u32> + '_ {
        self.by_node.get(&node).into_iter().flatten().copied()
    }

    fn touches(&self, road: u32, node: i64) -> bool {
        let road = &self.roads[road as usize];
        road.from_node() == node || road.to_node() == node
    }
}
