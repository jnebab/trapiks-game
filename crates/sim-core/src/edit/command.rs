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
    BuildRoundabout {
        node: u32,
        radius_m: u8,
    },
    AddRoad {
        from: Endpoint,
        to: Endpoint,
        via: Option<[f64; 2]>,
        lanes_forward: u8,
        lanes_backward: u8,
        layer: i8,
    },
    Undo,
    SetDemand {
        vehicles_per_hour: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Endpoint {
    Node { node: u32 },
    OnRoad { road: u32, at_m: f64 },
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
    InvalidRadius,
    AlreadyRoundabout,
    RoundaboutTooLarge,
    LayerMismatch,
    RoundaboutTooTight,
    InvalidLayer,
    EndpointIsolated,
    TooCloseToEnd,
    SameEndpoint,
    InvalidLength,
    AngleTooSharp,
    CrossesRoad,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QuoteOutcome {
    Ok(i64),
    Err(EditError),
}
