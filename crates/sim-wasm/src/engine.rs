use std::rc::Rc;

use trapiks_sim_core::config::SimConfig;
use trapiks_sim_core::consts::MAX_VEHICLES;
use trapiks_sim_core::edit::{BudgetState, CommandResult, EditCommand, Outcome};
use trapiks_sim_core::render::{
    ApproachMarkers, JunctionShapes, SignalPills, approach_markers, area_render, junction_shapes,
    network_delta, node_render, road_render, road_setbacks, signal_pills, signal_states,
};
use trapiks_sim_core::sim::{Sim, Snapshot};
use wasm_bindgen::prelude::*;

use crate::delta::DeltaGeometry;
use crate::geometry::{AreaGeometry, NodeGeometry, RoadGeometry};
use crate::map_handle::{LoadedMap, MapHandle, decode};
use crate::shared::{from_js, js_error, to_js};
use crate::signals::SignalPillGeometry;
use crate::snapshot::SnapshotPointers;
use crate::street::{ApproachMarkerGeometry, JunctionShapeGeometry};

#[wasm_bindgen]
pub struct Engine {
    loaded: Rc<LoadedMap>,
    sim: Sim,
    snapshot: Snapshot,
    setbacks: Vec<f32>,
    shapes: JunctionShapes,
    markers: ApproachMarkers,
    pills: SignalPills,
}

fn command_of(value: JsValue) -> Result<EditCommand, JsError> {
    from_js(value)
}

fn first_failure(results: &[CommandResult]) -> Option<String> {
    results.iter().find_map(|result| match &result.outcome {
        Outcome::Err(error) => Some(format!("{error:?}")),
        Outcome::Ok(_) => None,
    })
}

fn preallocated_snapshot() -> Snapshot {
    Snapshot {
        ids: Vec::with_capacity(MAX_VEHICLES),
        x: Vec::with_capacity(MAX_VEHICLES),
        y: Vec::with_capacity(MAX_VEHICLES),
        heading: Vec::with_capacity(MAX_VEHICLES),
        style: Vec::with_capacity(MAX_VEHICLES),
        layer: Vec::with_capacity(MAX_VEHICLES),
    }
}

#[wasm_bindgen]
impl Engine {
    pub fn load(bytes: &[u8], config: JsValue) -> Result<Engine, JsError> {
        Engine::build(Rc::new(decode(bytes)?), config)
    }

    #[wasm_bindgen(js_name = fromMap)]
    pub fn from_map(map: &MapHandle, config: JsValue) -> Result<Engine, JsError> {
        Engine::build(map.loaded(), config)
    }

    fn build(loaded: Rc<LoadedMap>, config: JsValue) -> Result<Engine, JsError> {
        let config: SimConfig = from_js(config)?;
        let sim = Sim::from_config(&loaded.map, &config);
        Ok(Engine {
            setbacks: road_setbacks(sim.network()),
            shapes: junction_shapes(sim.network()),
            markers: approach_markers(sim.network()),
            pills: signal_pills(sim.network()),
            loaded,
            sim,
            snapshot: preallocated_snapshot(),
        })
    }

    pub fn meta(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.loaded.meta).map_err(js_error)
    }

    #[wasm_bindgen(js_name = commandLog)]
    pub fn command_log(&self) -> Result<JsValue, JsError> {
        to_js(&self.sim.command_log())
    }

    pub fn replay(&mut self, commands: JsValue) -> Result<JsValue, JsError> {
        let commands: Vec<EditCommand> = from_js(commands)?;
        for command in commands {
            self.sim.enqueue(command);
        }
        self.sim.apply_queued();
        let results = self.sim.take_results();
        match first_failure(&results) {
            Some(error) => Err(js_error(error)),
            None => to_js(&results),
        }
    }

    #[wasm_bindgen(js_name = roadGeometry)]
    pub fn road_geometry(&self) -> RoadGeometry {
        RoadGeometry::from(road_render(&self.loaded.map))
    }

    #[wasm_bindgen(js_name = nodeGeometry)]
    pub fn node_geometry(&self) -> NodeGeometry {
        NodeGeometry::from(node_render(&self.loaded.map))
    }

    #[wasm_bindgen(js_name = areaGeometry)]
    pub fn area_geometry(&self) -> AreaGeometry {
        AreaGeometry::from(area_render(&self.loaded.map))
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

    #[wasm_bindgen(js_name = inspectRoad)]
    pub fn inspect_road(&self, road: u32) -> Result<JsValue, JsError> {
        to_js(&self.sim.inspect_road(road))
    }

    #[wasm_bindgen(js_name = inspectNode)]
    pub fn inspect_node(&self, node: u32) -> Result<JsValue, JsError> {
        to_js(&self.sim.inspect_node(node))
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
