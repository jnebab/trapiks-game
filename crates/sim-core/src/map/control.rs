use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Control {
    Priority,
    Yield,
    Stop,
    AllWayStop,
    Signal,
}

impl Control {
    pub const ALL: [Control; 5] = [
        Self::Priority,
        Self::Yield,
        Self::Stop,
        Self::AllWayStop,
        Self::Signal,
    ];

    pub fn code(self) -> u8 {
        match self {
            Self::Priority => 0,
            Self::Yield => 1,
            Self::Stop => 2,
            Self::AllWayStop => 3,
            Self::Signal => 4,
        }
    }
}
