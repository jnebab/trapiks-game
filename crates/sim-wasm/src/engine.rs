use trapiks_sim_core::config::SimConfig;
use trapiks_sim_core::consts::MAX_VEHICLES;
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::render::{
    ApproachMarkers, JunctionShapes, approach_markers, area_render, junction_shapes, node_render,
    road_render, road_setbacks,
};
use trapiks_sim_core::sim::{Sim, Snapshot};
use trapiks_sim_core::{MapMeta, map_meta};
use wasm_bindgen::prelude::*;

use crate::geometry::{AreaGeometry, NodeGeometry, RoadGeometry};
use crate::snapshot::SnapshotPointers;
use crate::street::{ApproachMarkerGeometry, JunctionShapeGeometry};

#[wasm_bindgen]
pub struct Engine {
    map: MapData,
    meta: MapMeta,
    sim: Sim,
    snapshot: Snapshot,
    setbacks: Vec<f32>,
    shapes: JunctionShapes,
    markers: ApproachMarkers,
}

fn js_error(error: impl ToString) -> JsError {
    JsError::new(&error.to_string())
}

fn preallocated_snapshot() -> Snapshot {
    Snapshot {
        ids: Vec::with_capacity(MAX_VEHICLES),
        x: Vec::with_capacity(MAX_VEHICLES),
        y: Vec::with_capacity(MAX_VEHICLES),
        heading: Vec::with_capacity(MAX_VEHICLES),
        style: Vec::with_capacity(MAX_VEHICLES),
    }
}

#[wasm_bindgen]
impl Engine {
    pub fn load(bytes: &[u8], config: JsValue) -> Result<Engine, JsError> {
        let config: SimConfig = serde_wasm_bindgen::from_value(config).map_err(js_error)?;
        let map = from_bytes(bytes).map_err(js_error)?;
        let meta = map_meta(&map, bytes);
        let sim = Sim::from_config(&map, &config);
        Ok(Engine {
            setbacks: road_setbacks(sim.network()),
            shapes: junction_shapes(sim.network()),
            markers: approach_markers(sim.network()),
            map,
            meta,
            sim,
            snapshot: preallocated_snapshot(),
        })
    }

    pub fn meta(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.meta).map_err(js_error)
    }

    #[wasm_bindgen(js_name = roadGeometry)]
    pub fn road_geometry(&self) -> RoadGeometry {
        RoadGeometry::from(road_render(&self.map))
    }

    #[wasm_bindgen(js_name = nodeGeometry)]
    pub fn node_geometry(&self) -> NodeGeometry {
        NodeGeometry::from(node_render(&self.map))
    }

    #[wasm_bindgen(js_name = areaGeometry)]
    pub fn area_geometry(&self) -> AreaGeometry {
        AreaGeometry::from(area_render(&self.map))
    }

    #[wasm_bindgen(js_name = roadSetbacks)]
    pub fn road_setbacks(&self) -> Vec<f32> {
        self.setbacks.clone()
    }

    #[wasm_bindgen(js_name = junctionShapes)]
    pub fn junction_shapes(&self) -> JunctionShapeGeometry {
        JunctionShapeGeometry::from(self.shapes.clone())
    }

    #[wasm_bindgen(js_name = approachMarkers)]
    pub fn approach_markers(&self) -> ApproachMarkerGeometry {
        ApproachMarkerGeometry::from(self.markers.clone())
    }

    pub fn step(&mut self) {
        self.sim.step();
    }

    pub fn tick(&self) -> f64 {
        self.sim.tick() as f64
    }

    #[wasm_bindgen(js_name = fillSnapshot)]
    pub fn fill_snapshot(&mut self) -> u32 {
        self.sim.fill_snapshot(&mut self.snapshot);
        debug_assert!(self.snapshot.ids.len() <= self.snapshot.ids.capacity());
        self.snapshot.ids.len() as u32
    }

    #[wasm_bindgen(js_name = snapshotPointers)]
    pub fn snapshot_pointers(&self) -> SnapshotPointers {
        SnapshotPointers::of(&self.snapshot)
    }

    pub fn stats(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.sim.stats()).map_err(js_error)
    }

    #[wasm_bindgen(js_name = roadSpeedRatio)]
    pub fn road_speed_ratio(&self) -> Vec<f32> {
        let mut out = Vec::new();
        self.sim.road_speed_ratio(&mut out);
        out
    }
}
