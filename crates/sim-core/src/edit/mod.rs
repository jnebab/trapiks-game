mod apply;
mod budget;
mod command;
pub mod cost;
mod flyover;
mod flyover_rules;
mod inspect;
mod roundabout;
mod validate;
pub mod vehicles;

pub use apply::{Edit, Scope};
pub use budget::{Budget, BudgetState};
pub use command::{CommandResult, EditCommand, EditError, EditOutcome, Outcome, QuoteOutcome};
pub use flyover::{FlyoverUndo, restore_through, truncate as truncate_flyover};
pub use inspect::{
    NodeInspection, RoadInspection, SignalInfo, TurnInfo, inspect_node, inspect_road,
};
pub use roundabout::{
    ArmRecord, ROUNDABOUT_MAX_RADIUS, ROUNDABOUT_MIN_CHORD, ROUNDABOUT_MIN_RADIUS, RoundaboutUndo,
    restore_arms, truncate as truncate_roundabout,
};
pub use validate::{MAX_VPH, Prepared, prepare};

#[derive(Clone, Debug, PartialEq)]
pub struct UndoEntry {
    pub inverse: Edit,
    pub cost: i64,
}
