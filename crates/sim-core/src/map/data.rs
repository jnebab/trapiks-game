use serde::{Deserialize, Serialize};

use super::{Control, RoadClass};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct GeoOrigin {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct NodeTable {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub control: Vec<Control>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct RoadTable {
    pub from: Vec<u32>,
    pub to: Vec<u32>,
    pub class: Vec<RoadClass>,
    pub lanes_forward: Vec<u8>,
    pub lanes_backward: Vec<u8>,
    pub speed_kph: Vec<u8>,
    pub layer: Vec<i8>,
    pub name: Vec<u32>,
    pub point_start: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PointTable {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct TurnBan {
    pub via_node: u32,
    pub from_road: u32,
    pub to_road: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MapData {
    pub origin: GeoOrigin,
    pub nodes: NodeTable,
    pub roads: RoadTable,
    pub points: PointTable,
    pub names: Vec<String>,
    pub turn_bans: Vec<TurnBan>,
}

impl MapData {
    pub fn road_count(&self) -> usize {
        self.roads.from.len()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.x.len()
    }

    pub fn road_points(&self, road: u32) -> (&[f32], &[f32]) {
        let index = road as usize;
        let start = self.roads.point_start[index] as usize;
        let end = self.roads.point_start[index + 1] as usize;
        (&self.points.x[start..end], &self.points.y[start..end])
    }
}
