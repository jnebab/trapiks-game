use std::rc::Rc;

use serde::Serialize;
use trapiks_sim_core::challenge::Challenge;
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::{MapMeta, map_meta};
use wasm_bindgen::prelude::*;

use crate::shared::{from_js, js_error, to_js};

pub struct LoadedMap {
    pub map: MapData,
    pub meta: MapMeta,
}

#[wasm_bindgen]
pub struct MapHandle {
    loaded: Rc<LoadedMap>,
}

#[derive(Serialize)]
struct CenterXy {
    x: f64,
    y: f64,
}

impl MapHandle {
    pub fn loaded(&self) -> Rc<LoadedMap> {
        Rc::clone(&self.loaded)
    }
}

pub fn decode(bytes: &[u8]) -> Result<LoadedMap, JsError> {
    let map = from_bytes(bytes).map_err(js_error)?;
    let meta = map_meta(&map, bytes);
    Ok(LoadedMap { map, meta })
}

#[wasm_bindgen]
impl MapHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<MapHandle, JsError> {
        Ok(MapHandle {
            loaded: Rc::new(decode(bytes)?),
        })
    }

    #[wasm_bindgen(js_name = challengeCenter)]
    pub fn challenge_center(&self, challenge: JsValue) -> Result<JsValue, JsError> {
        let challenge: Challenge = from_js(challenge)?;
        let (x, y) = challenge.center_xy(&self.loaded.map);
        to_js(&CenterXy { x, y })
    }
}
