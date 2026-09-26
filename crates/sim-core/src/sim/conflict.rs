use crate::consts::{CONFLICT_STOP_BACK, STOPPED_SPEED};
use crate::network::{Conflict, Junction, Movement};
use crate::vehicle::approach::{PriorityKey, RuleContext, movement_priority};
use crate::vehicle::{Place, movement_key};

use super::Sim;

struct InMovement<'a> {
    slot: u32,
    node: u32,
    junction: &'a Junction,
    movement: &'a Movement,
    conflicts: &'a [Conflict],
    s: f64,
}

impl Sim {
    pub(super) fn conflict_gap(&self, ctx: &RuleContext, slot: u32) -> Option<f64> {
        let at = self.movement_position(ctx, slot)?;
        let mut own = None;
        let mut best: Option<f64> = None;
        for conflict in at.conflicts {
            let stop = conflict.s_self - CONFLICT_STOP_BACK;
            if conflict.merge || at.s >= stop || !point_held(ctx, &at, conflict, &mut own) {
                continue;
            }
            let gap = stop - at.s;
            best = Some(best.map_or(gap, |b| b.min(gap)));
        }
        best
    }

    fn movement_position(&self, ctx: &RuleContext, slot: u32) -> Option<InMovement<'_>> {
        let index = slot as usize;
        let on_link = matches!(self.vehicles.place[index], Place::Link { .. });
        if on_link && !self.vehicles.committed[index] {
            return None;
        }
        let ahead = self.ahead_of(slot);
        let (node, movement) = ahead.crossing()?;
        if !others_at(ctx, node, slot) {
            return None;
        }
        let s = self.vehicles.s[index];
        let junction = self.network.junction(node)?;
        let movement = usize::from(movement);
        Some(InMovement {
            slot,
            node,
            junction,
            movement: junction.movements.get(movement)?,
            conflicts: junction.conflicts.get(movement)?,
            s: if on_link { s - ahead.end()? } else { s },
        })
    }
}

fn others_at(ctx: &RuleContext, node: u32, slot: u32) -> bool {
    ctx.occupancy
        .at_node(node)
        .iter()
        .any(|entry| entry.slot != slot)
}

fn point_held(
    ctx: &RuleContext,
    at: &InMovement,
    conflict: &Conflict,
    own: &mut Option<PriorityKey>,
) -> bool {
    let Some(movement) = at.junction.movements.get(usize::from(conflict.other)) else {
        return false;
    };
    let low = conflict.s_other - CONFLICT_STOP_BACK;
    for entry in ctx.occupancy.range(movement_key(at.node, conflict.other)) {
        let high = conflict.s_other + ctx.vehicles.length(entry.slot) + CONFLICT_STOP_BACK;
        if entry.s < low || entry.s > high {
            continue;
        }
        if ctx.vehicles.v[entry.slot as usize] < STOPPED_SPEED {
            return true;
        }
        let own = *own.get_or_insert_with(|| movement_priority(ctx, at.slot, at.node, at.movement));
        if movement_priority(ctx, entry.slot, at.node, movement) < own {
            return true;
        }
    }
    false
}
