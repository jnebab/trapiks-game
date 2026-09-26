use crate::map::Control;
use crate::network::Network;

const DELETE_PER_100M: i64 = 50;
const DELETE_MIN: i64 = 100;
const LANE_ADDED_PER_100M: i64 = 40;
const LANE_REMOVED_PER_100M: i64 = 10;
const ONE_WAY_SWITCH: i64 = 50;
const SPEED_LIMIT: i64 = 10;
const SIGNAL_TIMING: i64 = 20;
const TURN_RULE: i64 = 20;

fn len100(network: &Network, road: u32) -> i64 {
    let hundreds = (network.roads.length[road as usize] / 100.0).round();
    (hundreds as i64).max(1)
}

pub fn delete_road(network: &Network, road: u32) -> i64 {
    (DELETE_PER_100M * len100(network, road)).max(DELETE_MIN)
}

pub fn set_lanes(network: &Network, road: u32, forward: u8, backward: u8) -> i64 {
    let index = road as usize;
    let old = (
        network.roads.lanes_forward[index],
        network.roads.lanes_backward[index],
    );
    let added = i64::from(forward.saturating_sub(old.0) + backward.saturating_sub(old.1));
    let removed = i64::from(old.0.saturating_sub(forward) + old.1.saturating_sub(backward));
    let per_lane =
        (LANE_ADDED_PER_100M * added + LANE_REMOVED_PER_100M * removed) * len100(network, road);
    let switch = if is_two_way(old) == is_two_way((forward, backward)) {
        0
    } else {
        ONE_WAY_SWITCH
    };
    per_lane + switch
}

fn is_two_way((forward, backward): (u8, u8)) -> bool {
    forward > 0 && backward > 0
}

pub fn speed_limit() -> i64 {
    SPEED_LIMIT
}

pub fn junction_control(control: Control) -> i64 {
    match control {
        Control::Signal => 500,
        Control::AllWayStop => 80,
        Control::Stop | Control::Priority => 50,
        Control::Yield => 30,
    }
}

pub fn signal_timing() -> i64 {
    SIGNAL_TIMING
}

pub fn turn_rule() -> i64 {
    TURN_RULE
}
