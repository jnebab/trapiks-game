use crate::consts::TICKS_PER_SECOND;
use crate::map::{Control, TurnBan};
use crate::network::{Network, Timing, cycle_ticks, road_of};

use super::apply::Edit;
use super::{EditCommand, EditError, cost};

pub const MAX_VPH: f64 = 200_000.0;
const MAX_LANES: u8 = 8;
const MIN_KPH: u8 = 10;
const MAX_KPH: u8 = 120;
const MIN_GREEN_S: u32 = 5;
const MAX_GREEN_S: u32 = 120;
const MIN_JUNCTION_DEGREE: usize = 3;

#[derive(Clone, Debug, PartialEq)]
pub enum Prepared {
    Edit { edit: Edit, cost: i64 },
    Undo,
    Demand { vehicles_per_hour: f64 },
}

type Planned = Result<Prepared, EditError>;

pub fn prepare(network: &Network, command: &EditCommand, can_undo: bool) -> Planned {
    match *command {
        EditCommand::DeleteRoad { road } => delete_road(network, road),
        EditCommand::SetLanes {
            road,
            forward,
            backward,
        } => set_lanes(network, road, forward, backward),
        EditCommand::SetSpeedLimit { road, kph } => set_speed(network, road, kph),
        EditCommand::SetJunctionControl { node, control } => set_control(network, node, control),
        EditCommand::SetSignalTiming {
            node,
            ref greens_s,
            offset_s,
        } => set_timing(network, node, greens_s, offset_s),
        EditCommand::SetTurnAllowed {
            node,
            from_road,
            to_road,
            allowed,
        } => set_turn(
            network,
            TurnBan {
                via_node: node,
                from_road,
                to_road,
            },
            allowed,
        ),
        EditCommand::Undo => can_undo
            .then_some(Prepared::Undo)
            .ok_or(EditError::NothingToUndo),
        EditCommand::SetDemand { vehicles_per_hour } => set_demand(network, vehicles_per_hour),
    }
}

fn edit(edit: Edit, cost: i64) -> Planned {
    Ok(Prepared::Edit { edit, cost })
}

fn require(condition: bool, error: EditError) -> Result<(), EditError> {
    if condition { Ok(()) } else { Err(error) }
}

fn check_road(network: &Network, road: u32) -> Result<(), EditError> {
    require(
        (road as usize) < network.roads.count(),
        EditError::RoadNotFound,
    )?;
    require(network.roads.is_live(road), EditError::RoadDeleted)?;
    let outside = network
        .region()
        .is_some_and(|mask| !mask.active_road[road as usize]);
    require(!outside, EditError::OutsideRegion)
}

fn check_node(network: &Network, node: u32) -> Result<(), EditError> {
    require(
        (node as usize) < network.nodes.count(),
        EditError::NodeNotFound,
    )?;
    let outside = network.region().is_some() && network.active_degree(node) == 0;
    require(!outside, EditError::OutsideRegion)
}

fn delete_road(network: &Network, road: u32) -> Planned {
    check_road(network, road)?;
    let change = Edit::SetDeleted {
        road,
        deleted: true,
    };
    edit(change, cost::delete_road(network, road))
}

fn set_lanes(network: &Network, road: u32, forward: u8, backward: u8) -> Planned {
    check_road(network, road)?;
    let total = u16::from(forward) + u16::from(backward);
    let valid = total >= 1 && forward <= MAX_LANES && backward <= MAX_LANES;
    require(valid, EditError::InvalidLanes)?;
    let index = road as usize;
    let current = (
        network.roads.lanes_forward[index],
        network.roads.lanes_backward[index],
    );
    require(current != (forward, backward), EditError::NoChange)?;
    let cost = cost::set_lanes(network, road, forward, backward);
    let change = Edit::SetLanes {
        road,
        forward,
        backward,
    };
    edit(change, cost)
}

fn set_speed(network: &Network, road: u32, kph: u8) -> Planned {
    check_road(network, road)?;
    require((MIN_KPH..=MAX_KPH).contains(&kph), EditError::InvalidSpeed)?;
    let current = network.roads.speed_kph[road as usize];
    require(current != kph, EditError::NoChange)?;
    edit(Edit::SetSpeed { road, kph }, cost::speed_limit())
}

fn set_control(network: &Network, node: u32, control: Control) -> Planned {
    check_node(network, node)?;
    let degree = network.active_degree(node);
    require(degree >= MIN_JUNCTION_DEGREE, EditError::NotAJunction)?;
    let current = network.nodes.control[node as usize];
    require(current != control, EditError::NoChange)?;
    let cost = cost::junction_control(control);
    edit(Edit::SetControl { node, control }, cost)
}

fn set_timing(network: &Network, node: u32, greens_s: &[u32], offset_s: u32) -> Planned {
    check_node(network, node)?;
    let cluster = network
        .signals()
        .cluster_at(node)
        .ok_or(EditError::NotSignalized)?;
    let timing = timing_from_seconds(greens_s, offset_s);
    let phases_match = greens_s.len() == cluster.phases.len();
    let greens_valid = greens_s
        .iter()
        .all(|green| (MIN_GREEN_S..=MAX_GREEN_S).contains(green));
    let offset_valid = u64::from(timing.offset_ticks) < cycle_ticks(&timing.green_ticks);
    require(
        phases_match && greens_valid && offset_valid,
        EditError::InvalidTiming,
    )?;
    require(cluster.timing() != timing, EditError::NoChange)?;
    let change = Edit::SetTiming {
        key: cluster.key(),
        timing: Some(timing),
    };
    edit(change, cost::signal_timing())
}

fn timing_from_seconds(greens_s: &[u32], offset_s: u32) -> Timing {
    Timing {
        green_ticks: greens_s
            .iter()
            .map(|&green| green.saturating_mul(TICKS_PER_SECOND))
            .collect(),
        offset_ticks: offset_s.saturating_mul(TICKS_PER_SECOND),
    }
}

fn set_turn(network: &Network, ban: TurnBan, allowed: bool) -> Planned {
    check_node(network, ban.via_node)?;
    let exists = network
        .turns_ignoring_bans(ban.via_node)
        .iter()
        .any(|&(from, to, _)| road_of(from) == ban.from_road && road_of(to) == ban.to_road);
    require(exists, EditError::TurnNotFound)?;
    let banned = !allowed;
    let current = network.is_banned(ban.via_node, ban.from_road, ban.to_road);
    require(current != banned, EditError::NoChange)?;
    edit(Edit::SetBan { ban, banned }, cost::turn_rule())
}

fn set_demand(network: &Network, vehicles_per_hour: f64) -> Planned {
    require(network.region().is_none(), EditError::DemandLocked)?;
    let valid = vehicles_per_hour.is_finite() && (0.0..=MAX_VPH).contains(&vehicles_per_hour);
    require(valid, EditError::InvalidDemand)?;
    Ok(Prepared::Demand { vehicles_per_hour })
}
