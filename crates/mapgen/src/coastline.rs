mod boundary;
mod clip;
mod walk;

use std::collections::BTreeMap;

use crate::input::{OsmNode, OsmWay};
use crate::osm::LatLon;

pub use boundary::LonLatBox;

type Point = (f64, f64);

pub fn sea_rings(
    ways: &[&OsmWay],
    nodes: &BTreeMap<i64, OsmNode>,
    bbox: &LonLatBox,
) -> (Vec<Vec<LatLon>>, u32) {
    let chains = join_chains(way_points(ways, nodes));
    let open: Vec<Vec<Point>> = chains
        .into_iter()
        .filter_map(|chain| open_chain(chain, bbox))
        .collect();
    let (pieces, clip_dropped) = clip::clip_all(&open, bbox);
    let (rings, walk_dropped) = walk::sea_polygons(&pieces, bbox);
    let rings = rings.into_iter().map(to_lat_lon).collect();
    (rings, clip_dropped + walk_dropped)
}

fn way_points(ways: &[&OsmWay], nodes: &BTreeMap<i64, OsmNode>) -> Vec<Vec<Point>> {
    ways.iter()
        .filter_map(|way| {
            way.nodes
                .iter()
                .map(|id| nodes.get(id).map(|n| (n.lon, n.lat)))
                .collect::<Option<Vec<Point>>>()
        })
        .filter(|points| points.len() >= 2)
        .collect()
}

fn join_chains(ways: Vec<Vec<Point>>) -> Vec<Vec<Point>> {
    let mut pending: Vec<Option<Vec<Point>>> = ways.into_iter().map(Some).collect();
    let mut chains = Vec::new();
    for index in 0..pending.len() {
        let Some(mut chain) = pending[index].take() else {
            continue;
        };
        while !is_closed(&chain) && grow(&mut chain, &mut pending) {}
        chains.push(chain);
    }
    chains
}

fn is_closed(chain: &[Point]) -> bool {
    chain.len() >= 3 && chain.first() == chain.last()
}

fn grow(chain: &mut Vec<Point>, pending: &mut [Option<Vec<Point>>]) -> bool {
    let (first, last) = (chain[0], chain[chain.len() - 1]);
    if let Some(next) = take_where(pending, |way| way[0] == last) {
        chain.extend_from_slice(&next[1..]);
        return true;
    }
    if let Some(mut previous) = take_where(pending, |way| way[way.len() - 1] == first) {
        previous.extend_from_slice(&chain[1..]);
        *chain = previous;
        return true;
    }
    false
}

fn take_where(
    pending: &mut [Option<Vec<Point>>],
    matches: impl Fn(&[Point]) -> bool,
) -> Option<Vec<Point>> {
    let slot = pending
        .iter_mut()
        .find(|slot| slot.as_deref().is_some_and(&matches))?;
    slot.take()
}

fn open_chain(chain: Vec<Point>, bbox: &LonLatBox) -> Option<Vec<Point>> {
    if !is_closed(&chain) {
        return Some(chain);
    }
    let ring = &chain[..chain.len() - 1];
    let outside = ring.iter().position(|p| !bbox.contains(*p))?;
    let mut rotated: Vec<Point> = ring[outside..]
        .iter()
        .chain(&ring[..outside])
        .copied()
        .collect();
    rotated.push(ring[outside]);
    Some(rotated)
}

fn to_lat_lon(ring: Vec<Point>) -> Vec<LatLon> {
    ring.into_iter()
        .map(|(lon, lat)| LatLon { lat, lon })
        .collect()
}
