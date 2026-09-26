mod apply;
mod budget;
mod command;
pub mod cost;
mod validate;
pub mod vehicles;

pub use apply::{Edit, Scope};
pub use budget::Budget;
pub use command::{CommandResult, EditCommand, EditError, EditOutcome, Outcome};
pub use validate::{MAX_VPH, Prepared, prepare};

#[derive(Clone, Debug, PartialEq)]
pub struct UndoEntry {
    pub inverse: Edit,
    pub cost: i64,
}
