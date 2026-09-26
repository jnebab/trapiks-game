use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

pub fn js_error(error: impl ToString) -> JsError {
    JsError::new(&error.to_string())
}

pub fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(js_error)
}

pub fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_err(js_error)
}
