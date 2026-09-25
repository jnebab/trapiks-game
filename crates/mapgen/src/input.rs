use std::collections::BTreeMap;

use crate::osm::{Element, LatLon, Member, Response, Tags};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsmNode {
    pub lat: f64,
    pub lon: f64,
    pub tags: Tags,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsmWay {
    pub nodes: Vec<i64>,
    pub tags: Tags,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsmRelation {
    pub members: Vec<Member>,
    pub tags: Tags,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsmData {
    pub nodes: BTreeMap<i64, OsmNode>,
    pub ways: BTreeMap<i64, OsmWay>,
    pub relations: BTreeMap<i64, OsmRelation>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadStats {
    pub dropped_ways: usize,
}

pub fn parse(bytes: &[u8]) -> Result<Response, String> {
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}

pub fn load(paths: &[String]) -> Result<(OsmData, LoadStats), String> {
    let mut data = OsmData::default();
    let mut stats = LoadStats::default();
    for path in paths {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let response = parse(&bytes).map_err(|e| format!("{path}: {e}"))?;
        merge(&mut data, &mut stats, response);
    }
    Ok((data, stats))
}

pub fn merge(data: &mut OsmData, stats: &mut LoadStats, response: Response) {
    for element in response.elements {
        merge_element(data, stats, element);
    }
}

fn merge_element(data: &mut OsmData, stats: &mut LoadStats, element: Element) {
    match element {
        Element::Node { id, lat, lon, tags } => {
            data.nodes.insert(id, OsmNode { lat, lon, tags });
        }
        Element::Way {
            id,
            nodes,
            geometry,
            tags,
        } => merge_way(data, stats, id, OsmWay { nodes, tags }, geometry),
        Element::Relation { id, members, tags } => {
            data.relations.insert(id, OsmRelation { members, tags });
        }
    }
}

fn merge_way(
    data: &mut OsmData,
    stats: &mut LoadStats,
    id: i64,
    way: OsmWay,
    geometry: Option<Vec<LatLon>>,
) {
    let geometry = geometry.unwrap_or_default();
    if !geometry.is_empty() && geometry.len() != way.nodes.len() {
        stats.dropped_ways += 1;
        return;
    }
    for (node, point) in way.nodes.iter().zip(&geometry) {
        data.nodes.entry(*node).or_insert_with(|| OsmNode {
            lat: point.lat,
            lon: point.lon,
            tags: Tags::new(),
        });
    }
    data.ways.insert(id, way);
}
