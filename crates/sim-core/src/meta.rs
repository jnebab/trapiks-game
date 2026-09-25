use serde::Serialize;
use ts_rs::TS;

use crate::map::{AreaKind, Control, MapData, PointTable, RoadClass, map_hash};

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct MapMeta {
    pub map_hash: String,
    pub road_count: u32,
    pub node_count: u32,
    pub area_ring_count: u32,
    pub class_names: Vec<RoadClass>,
    pub control_names: Vec<Control>,
    pub area_kind_names: Vec<AreaKind>,
    pub names: Vec<String>,
    pub bounds: [f32; 4],
}

pub fn map_meta(map: &MapData, bytes: &[u8]) -> MapMeta {
    MapMeta {
        map_hash: format!("{:016x}", map_hash(bytes)),
        road_count: count(map.road_count()),
        node_count: count(map.node_count()),
        area_ring_count: count(map.areas.kind.len()),
        class_names: RoadClass::ALL.to_vec(),
        control_names: Control::ALL.to_vec(),
        area_kind_names: AreaKind::ALL.to_vec(),
        names: map.names.clone(),
        bounds: point_bounds(&map.points),
    }
}

fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn point_bounds(points: &PointTable) -> [f32; 4] {
    if points.x.is_empty() {
        return [0.0; 4];
    }
    let fold = |values: &[f32], pick: fn(f32, f32) -> f32, start: f32| {
        values.iter().copied().fold(start, pick)
    };
    [
        fold(&points.x, f32::min, f32::INFINITY),
        fold(&points.y, f32::min, f32::INFINITY),
        fold(&points.x, f32::max, f32::NEG_INFINITY),
        fold(&points.y, f32::max, f32::NEG_INFINITY),
    ]
}
