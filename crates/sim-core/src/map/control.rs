use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Control {
    Priority,
    Yield,
    Stop,
    AllWayStop,
    Signal,
}
