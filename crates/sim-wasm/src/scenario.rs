use trapiks_sim_core::fixtures::{
    FOUR_WAY_HASH_5000, GRID_CITY_HASH_1000, four_way_5000, grid_city_demand_1000,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = runScenario)]
pub fn run_scenario(name: &str) -> Result<String, JsError> {
    let (computed, expected) = match name {
        "four_way_5000" => (four_way_5000(), FOUR_WAY_HASH_5000),
        "grid_city_demand_1000" => (grid_city_demand_1000(), GRID_CITY_HASH_1000),
        _ => return Err(JsError::new(&format!("unknown scenario {name}"))),
    };
    Ok(format!("{computed:016x} {expected:016x}"))
}
