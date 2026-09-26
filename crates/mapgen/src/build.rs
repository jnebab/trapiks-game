mod area_table;
mod tables;

use std::collections::BTreeMap;

use trapiks_sim_core::map::{MapData, validate};

use crate::coastline::LonLatBox;
use crate::controls::Positions;
use crate::filter::road_class;
use crate::input::{OsmData, OsmWay};
use crate::restrictions;
use crate::tags::{self, Oneway};
use crate::topology::{self, Attributes, TopoRoad, WayInput};
use trapiks_sim_core::geo::Projection;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildStats {
    pub dropped_chains: u32,
    pub dropped_coastline_pieces: u32,
}

pub fn build(osm: &OsmData) -> Result<(MapData, BuildStats), String> {
    let ways: Vec<WayInput> = osm
        .ways
        .iter()
        .filter_map(|(id, way)| way_input(osm, *id, way))
        .collect();
    let topology = topology::build(&ways);
    let bbox = road_bbox(osm, &topology.roads)?;
    let projection = Projection::centered(bbox.min_lat, bbox.min_lon, bbox.max_lat, bbox.max_lon);
    let positions = project_nodes(osm, &topology.roads, &projection);
    let node_index: BTreeMap<i64, u32> = topology.nodes.iter().copied().zip(0u32..).collect();
    let (areas, stats) = area_table::area_table(osm, &bbox, &projection);
    let map = MapData {
        origin: projection.origin(),
        nodes: tables::node_table(osm, &topology, &positions),
        turn_bans: restrictions::expand(osm, &topology.roads, &node_index),
        areas,
        ..tables::road_tables(&topology.roads, &node_index, &positions)
    };
    validate(&map).map_err(|e| e.to_string())?;
    Ok((map, stats))
}

fn way_input(osm: &OsmData, id: i64, way: &OsmWay) -> Option<WayInput> {
    let class = road_class(&way.tags)?;
    let mut nodes = topology::clean_nodes(&way.nodes);
    if nodes.len() < 2 || nodes.iter().any(|node| !osm.nodes.contains_key(node)) {
        return None;
    }
    let oneway = tags::oneway(&way.tags, class);
    let (mut forward, mut backward) = tags::lanes(&way.tags, class, oneway);
    if oneway == Oneway::Backward {
        nodes.reverse();
        (forward, backward) = (backward, forward);
    }
    let attributes = Attributes {
        class,
        lanes_forward: forward,
        lanes_backward: backward,
        speed_kph: tags::speed_kph(&way.tags, class),
        layer: tags::layer(&way.tags),
        name: tags::name(&way.tags).map(str::to_string),
    };
    Some(WayInput {
        id,
        nodes,
        attributes,
    })
}

fn used_nodes(roads: &[TopoRoad]) -> impl Iterator<Item = i64> + '_ {
    roads.iter().flat_map(|road| road.node_ids.iter().copied())
}

fn road_bbox(osm: &OsmData, roads: &[TopoRoad]) -> Result<LonLatBox, String> {
    let mut coords = used_nodes(roads).filter_map(|id| osm.nodes.get(&id));
    let first = coords.next().ok_or("no drivable roads in input")?;
    let init = LonLatBox {
        min_lon: first.lon,
        min_lat: first.lat,
        max_lon: first.lon,
        max_lat: first.lat,
    };
    Ok(coords.fold(init, |b, n| LonLatBox {
        min_lon: b.min_lon.min(n.lon),
        min_lat: b.min_lat.min(n.lat),
        max_lon: b.max_lon.max(n.lon),
        max_lat: b.max_lat.max(n.lat),
    }))
}

fn project_nodes(osm: &OsmData, roads: &[TopoRoad], projection: &Projection) -> Positions {
    used_nodes(roads)
        .filter_map(|id| {
            osm.nodes
                .get(&id)
                .map(|n| (id, projection.project(n.lat, n.lon)))
        })
        .collect()
}
