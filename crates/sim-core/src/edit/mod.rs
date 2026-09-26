mod apply;
mod budget;
mod command;
pub mod cost;
mod flyover;
mod flyover_rules;
mod validate;
pub mod vehicles;

pub use apply::{Edit, Scope};
pub use budget::Budget;
pub use command::{CommandResult, EditCommand, EditError, EditOutcome, Outcome};
pub use flyover::{FlyoverUndo, restore_through, truncate as truncate_flyover};
pub use validate::{MAX_VPH, Prepared, prepare};

#[derive(Clone, Debug, PartialEq)]
pub struct UndoEntry {
    pub inverse: Edit,
    pub cost: i64,
}
