use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::render::{area_render, node_render, road_render};
use trapiks_sim_core::{MapMeta, map_meta};
use wasm_bindgen::prelude::*;

use crate::geometry::{AreaGeometry, NodeGeometry, RoadGeometry};

#[wasm_bindgen]
pub struct Engine {
    map: MapData,
    meta: MapMeta,
}

#[wasm_bindgen]
impl Engine {
    pub fn load(bytes: &[u8]) -> Result<Engine, JsError> {
        let map = from_bytes(bytes).map_err(|error| JsError::new(&error.to_string()))?;
        let meta = map_meta(&map, bytes);
        Ok(Engine { map, meta })
    }

    pub fn meta(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.meta).map_err(|error| JsError::new(&error.to_string()))
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
}
