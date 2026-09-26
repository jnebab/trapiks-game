use serde::Serialize;
use trapiks_sim_core::config::SimConfig;
use trapiks_sim_core::consts::MAX_VEHICLES;
use trapiks_sim_core::edit::{BudgetState, EditCommand};
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::render::{
    ApproachMarkers, JunctionShapes, SignalPills, approach_markers, area_render, junction_shapes,
    network_delta, node_render, road_render, road_setbacks, signal_pills, signal_states,
};
use trapiks_sim_core::sim::{Sim, Snapshot};
use trapiks_sim_core::{MapMeta, map_meta};
use wasm_bindgen::prelude::*;

use crate::delta::DeltaGeometry;
use crate::geometry::{AreaGeometry, NodeGeometry, RoadGeometry};
use crate::signals::SignalPillGeometry;
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
    pills: SignalPills,
}

fn js_error(error: impl ToString) -> JsError {
    JsError::new(&error.to_string())
}

fn command_of(value: JsValue) -> Result<EditCommand, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(js_error)
}

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_err(js_error)
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
            pills: signal_pills(sim.network()),
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

    #[wasm_bindgen(js_name = signalPills)]
    pub fn signal_pills(&self) -> SignalPillGeometry {
        SignalPillGeometry::from(self.pills.clone())
    }

    #[wasm_bindgen(js_name = signalStates)]
    pub fn signal_states(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pills.link.len());
        signal_states(
            self.sim.network(),
            &self.pills.link,
            self.sim.tick(),
            &mut out,
        );
        out
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

    pub fn enqueue(&mut self, command: JsValue) -> Result<u32, JsError> {
        Ok(self.sim.enqueue(command_of(command)?))
    }

    pub fn quote(&self, command: JsValue) -> Result<JsValue, JsError> {
        to_js(&self.sim.quote(&command_of(command)?))
    }

    #[wasm_bindgen(js_name = flushCommands)]
    pub fn flush_commands(&mut self) {
        self.sim.apply_queued();
    }

    #[wasm_bindgen(js_name = takeResults)]
    pub fn take_results(&mut self) -> Result<JsValue, JsError> {
        to_js(&self.sim.take_results())
    }

    pub fn budget(&self) -> Result<JsValue, JsError> {
        to_js(&BudgetState::from(self.sim.budget()))
    }

    pub fn delta(&mut self, roads: Vec<u32>, nodes: Vec<u32>) -> DeltaGeometry {
        let delta = network_delta(self.sim.network(), &roads, &nodes);
        self.pills = delta.pills.clone();
        DeltaGeometry::from(delta)
    }
}
