use serde::Serialize;
use ts_rs::TS;

pub const MAP_FORMAT_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct EngineInfo {
    pub name: String,
    pub map_format_version: u32,
}

pub fn engine_info() -> EngineInfo {
    EngineInfo {
        name: "Trapiks".to_string(),
        map_format_version: MAP_FORMAT_VERSION,
    }
}
