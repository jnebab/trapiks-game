use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::consts::{THROUGH_MAX_TURN_DEG, UTURN_MIN_TURN_DEG};
use crate::geom::{Vec2, signed_turn};
use crate::network::link::{LinkId, reverse};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TurnKind {
    Right,
    Through,
    Left,
    UTurn,
}

impl TurnKind {
    pub fn rank(self) -> u8 {
        match self {
            Self::Through => 3,
            Self::Right => 2,
            Self::Left => 1,
            Self::UTurn => 0,
        }
    }

    pub fn is_left_side(self) -> bool {
        matches!(self, Self::Left | Self::UTurn)
    }
}

pub fn classify(from_link: LinkId, to_link: LinkId, d0: Vec2, d3: Vec2) -> TurnKind {
    if to_link == reverse(from_link) {
        return TurnKind::UTurn;
    }
    let theta = signed_turn(d0, d3).to_degrees();
    if theta.abs() <= THROUGH_MAX_TURN_DEG {
        return TurnKind::Through;
    }
    if theta.abs() >= UTURN_MIN_TURN_DEG {
        return TurnKind::UTurn;
    }
    if theta > 0.0 {
        TurnKind::Right
    } else {
        TurnKind::Left
    }
}
