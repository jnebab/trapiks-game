use crate::consts::{CAR_LENGTH, CONFLICT_CLEAR_MARGIN, STOPPED_SPEED};
use crate::network::{Conflict, Movement};
use crate::vehicle::approach::{PriorityKey, movement_priority};
use crate::vehicle::{Place, movement_key};

use super::Sim;

struct InMovement<'a> {
    node: u32,
    index: usize,
    movement: &'a Movement,
    s: f64,
}

impl Sim {
    pub(super) fn conflict_gap(&self, slot: u32) -> Option<f64> {
        let at = self.movement_position(slot)?;
        let conflicts = self.network.junction(at.node)?.conflicts.get(at.index)?;
        let own = movement_priority(&self.rule_context(), slot, at.node, at.movement);
        conflicts
            .iter()
            .map(|c| (c, c.s_self - CONFLICT_CLEAR_MARGIN))
            .filter(|&(c, stop)| !c.merge && at.s < stop && self.point_held(at.node, c, own))
            .map(|(_, stop)| stop - at.s)
            .reduce(f64::min)
    }

    fn movement_position(&self, slot: u32) -> Option<InMovement<'_>> {
        let index = slot as usize;
        let s = self.vehicles.s[index];
        match self.vehicles.place[index] {
            Place::Movement { node, movement, .. } => {
                let junction = self.network.junction(node)?;
                let movement = usize::from(movement);
                Some(InMovement {
                    node,
                    index: movement,
                    movement: junction.movements.get(movement)?,
                    s,
                })
            }
            Place::Link { link, .. } => {
                if !self.vehicles.committed[index] {
                    return None;
                }
                let next = self.vehicles.route_link(slot, 1)?;
                let node = self.network.link_to(link);
                let junction = self.network.junction(node)?;
                let movement = junction.movement_index(link, next)?;
                Some(InMovement {
                    node,
                    index: movement,
                    movement: &junction.movements[movement],
                    s: s - self.network.link_span(link).1,
                })
            }
        }
    }

    fn point_held(&self, node: u32, conflict: &Conflict, own: PriorityKey) -> bool {
        let low = conflict.s_other - CONFLICT_CLEAR_MARGIN;
        let high = conflict.s_other + CAR_LENGTH + CONFLICT_CLEAR_MARGIN;
        let Some(movement) = self
            .network
            .junction(node)
            .and_then(|j| j.movements.get(usize::from(conflict.other)))
        else {
            return false;
        };
        let ctx = self.rule_context();
        self.occupancy
            .range(movement_key(node, conflict.other))
            .iter()
            .filter(|entry| entry.s >= low && entry.s <= high)
            .any(|entry| {
                self.vehicles.v[entry.slot as usize] < STOPPED_SPEED
                    || movement_priority(&ctx, entry.slot, node, movement) < own
            })
    }
}
