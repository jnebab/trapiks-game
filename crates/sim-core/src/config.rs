use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SimMode {
    City,
    Region {
        center_x: f64,
        center_y: f64,
        radius: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SimConfig {
    pub seed: u64,
    pub mode: SimMode,
    pub vehicles_per_hour: f64,
}
