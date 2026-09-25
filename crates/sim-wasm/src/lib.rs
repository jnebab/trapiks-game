use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = engineInfo)]
pub fn engine_info() -> Result<JsValue, JsError> {
    let info = trapiks_sim_core::engine_info();
    serde_wasm_bindgen::to_value(&info).map_err(|error| JsError::new(&error.to_string()))
}
