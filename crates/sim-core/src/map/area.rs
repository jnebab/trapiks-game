use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AreaKind {
    Water,
    Park,
}

impl AreaKind {
    pub const ALL: [AreaKind; 2] = [Self::Water, Self::Park];

    pub fn code(self) -> u8 {
        match self {
            Self::Water => 0,
            Self::Park => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AreaTable {
    pub kind: Vec<AreaKind>,
    pub hole: Vec<bool>,
    pub ring_start: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}
