use std::collections::BTreeMap;

use trapiks_sim_core::map::AreaKind;

use crate::input::{OsmNode, OsmRelation, OsmWay};
use crate::osm::{LatLon, Tags};
use crate::simplify::{TOLERANCE_M, simplify_ring};
use crate::tags::tag;
use trapiks_sim_core::geo::Projection;

pub fn area_kind(tags: &Tags) -> Option<AreaKind> {
    let water = tag(tags, "natural") == Some("water") || tag(tags, "waterway") == Some("riverbank");
    if water {
        return Some(AreaKind::Water);
    }
    let park = matches!(tag(tags, "leisure"), Some("park" | "golf_course"))
        || matches!(
            tag(tags, "landuse"),
            Some("grass" | "recreation_ground" | "cemetery" | "forest" | "meadow")
        );
    park.then_some(AreaKind::Park)
}

pub fn closed_way_ring(
    way: &OsmWay,
    nodes: &BTreeMap<i64, OsmNode>,
) -> Option<(AreaKind, Vec<LatLon>)> {
    let kind = area_kind(&way.tags)?;
    let ids = &way.nodes;
    if ids.len() < 4 || ids.first() != ids.last() {
        return None;
    }
    let ring = ids
        .iter()
        .map(|id| {
            nodes.get(id).map(|n| LatLon {
                lat: n.lat,
                lon: n.lon,
            })
        })
        .collect::<Option<Vec<LatLon>>>()?;
    Some((kind, ring))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AreaRings {
    pub outers: Vec<Vec<LatLon>>,
    pub inners: Vec<Vec<LatLon>>,
}

pub struct RelationRings {
    pub kind: AreaKind,
    pub rings: AreaRings,
    pub dropped: u32,
}

pub fn relation_rings(relation: &OsmRelation) -> Option<RelationRings> {
    if tag(&relation.tags, "type") != Some("multipolygon") {
        return None;
    }
    let kind = area_kind(&relation.tags)?;
    let (outers, dropped_outer) = join_rings(member_chains(relation, |role| {
        role == "outer" || role.is_empty()
    }));
    let (inners, dropped_inner) = join_rings(member_chains(relation, |role| role == "inner"));
    Some(RelationRings {
        kind,
        rings: AreaRings { outers, inners },
        dropped: dropped_outer + dropped_inner,
    })
}

fn member_chains(relation: &OsmRelation, keep: impl Fn(&str) -> bool) -> Vec<Vec<LatLon>> {
    relation
        .members
        .iter()
        .filter(|m| keep(&m.role))
        .filter_map(|m| m.geometry.clone())
        .filter(|geometry| geometry.len() >= 2)
        .collect()
}

fn join_rings(chains: Vec<Vec<LatLon>>) -> (Vec<Vec<LatLon>>, u32) {
    let mut pending: Vec<Vec<LatLon>> = chains;
    pending.reverse();
    let mut rings = Vec::new();
    let mut dropped = 0;
    while let Some(mut chain) = pending.pop() {
        while !is_closed(&chain) && extend(&mut chain, &mut pending) {}
        if is_closed(&chain) {
            rings.push(chain);
        } else {
            dropped += 1;
        }
    }
    (rings, dropped)
}

fn is_closed(chain: &[LatLon]) -> bool {
    chain.len() >= 4 && chain.first() == chain.last()
}

fn extend(chain: &mut Vec<LatLon>, pending: &mut Vec<Vec<LatLon>>) -> bool {
    let Some(&end) = chain.last() else {
        return false;
    };
    let touches = |c: &Vec<LatLon>| c.first() == Some(&end) || c.last() == Some(&end);
    let Some(position) = pending.iter().rposition(touches) else {
        return false;
    };
    let mut next = pending.remove(position);
    if next.first() != Some(&end) {
        next.reverse();
    }
    chain.extend_from_slice(&next[1..]);
    true
}

pub fn finish(rings: Vec<Vec<LatLon>>, projection: &Projection) -> Vec<Vec<(f32, f32)>> {
    rings
        .iter()
        .filter_map(|ring| finish_ring(&project_ring(ring, projection)))
        .collect()
}

pub fn project_ring(ring: &[LatLon], projection: &Projection) -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = ring
        .iter()
        .map(|p| projection.project(p.lat, p.lon))
        .collect();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
}

pub fn finish_ring(points: &[(f64, f64)]) -> Option<Vec<(f32, f32)>> {
    let simplified = simplify_ring(points, TOLERANCE_M)?;
    Some(
        simplified
            .into_iter()
            .map(|(x, y)| (x as f32, y as f32))
            .collect(),
    )
}
