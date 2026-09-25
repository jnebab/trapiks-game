use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AreaKind {
    Water,
    Park,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AreaTable {
    pub kind: Vec<AreaKind>,
    pub ring_start: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}
