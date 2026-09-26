use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::map::Control;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EditCommand {
    DeleteRoad {
        road: u32,
    },
    SetLanes {
        road: u32,
        forward: u8,
        backward: u8,
    },
    SetSpeedLimit {
        road: u32,
        kph: u8,
    },
    SetJunctionControl {
        node: u32,
        control: Control,
    },
    SetSignalTiming {
        node: u32,
        greens_s: Vec<u32>,
        offset_s: u32,
    },
    SetTurnAllowed {
        node: u32,
        from_road: u32,
        to_road: u32,
        allowed: bool,
    },
    BuildFlyover {
        node: u32,
        through: [u32; 2],
    },
    Undo,
    SetDemand {
        vehicles_per_hour: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EditError {
    RoadNotFound,
    RoadDeleted,
    NodeNotFound,
    OutsideRegion,
    InvalidLanes,
    InvalidSpeed,
    NotAJunction,
    NotSignalized,
    InvalidTiming,
    TurnNotFound,
    NoChange,
    InsufficientBudget,
    NothingToUndo,
    DemandLocked,
    InvalidDemand,
    RoadNotIncident,
    FlyoverNotStraight,
    FlyoverTooShort,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EditOutcome {
    pub cost: i64,
    pub changed_roads: Vec<u32>,
    pub changed_nodes: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Outcome {
    Ok(EditOutcome),
    Err(EditError),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CommandResult {
    pub seq: u32,
    pub outcome: Outcome,
}
