use crate::network::Network;

use super::{Candidate, TurnKind};

pub fn assign(network: &Network, candidates: &[Candidate]) -> Vec<(u8, u8)> {
    let mut result = Vec::with_capacity(candidates.len());
    for group in candidates.chunk_by(|a, b| a.from_link == b.from_link) {
        let lanes = network.link_lanes(group[0].from_link);
        let summary = Summary::of(group);
        result.extend(group.iter().map(|c| range(lanes, &summary, c.kind)));
    }
    result
}

struct Summary {
    through: bool,
    right: bool,
    left_side: bool,
}

impl Summary {
    fn of(group: &[Candidate]) -> Self {
        Self {
            through: group.iter().any(|c| c.kind == TurnKind::Through),
            right: group.iter().any(|c| c.kind == TurnKind::Right),
            left_side: group.iter().any(|c| c.kind.is_left_side()),
        }
    }
}

fn range(lanes: u8, summary: &Summary, kind: TurnKind) -> (u8, u8) {
    let last = lanes.saturating_sub(1);
    if lanes <= 1 {
        return (0, 0);
    }
    if summary.through {
        return with_through(last, kind);
    }
    if summary.right && summary.left_side {
        return split(lanes, kind);
    }
    (0, last)
}

fn with_through(last: u8, kind: TurnKind) -> (u8, u8) {
    match kind {
        TurnKind::Through => (0, last),
        TurnKind::Right => (0, 0),
        TurnKind::Left | TurnKind::UTurn => (last, last),
    }
}

fn split(lanes: u8, kind: TurnKind) -> (u8, u8) {
    let half = lanes / 2;
    if kind == TurnKind::Right {
        return (0, half - 1);
    }
    (half, lanes - 1)
}
