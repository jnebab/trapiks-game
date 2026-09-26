use super::Place;
use super::approach::RuleContext;
use super::idm::acceleration;
use super::leader::current_ahead;
use super::neighbours::{Follower, Probe, follower_on, leader_on};

pub const LC_SAFE_DECEL: f64 = 4.0;
pub const LC_POLITENESS: f64 = 0.3;
pub const LC_THRESHOLD: f64 = 0.1;
pub const LC_KEEP_RIGHT_BIAS: f64 = 0.3;
pub const LC_COOLDOWN_TICKS: u64 = 30;
pub const LC_ANIMATION_TICKS: u64 = 30;
pub const LC_MIN_DISTANCE_TO_END: f64 = 60.0;
pub const LC_DISCRETIONARY_EVERY: u64 = 5;

type Obstacle = Option<(f64, f64)>;

struct Eligible {
    probe: Probe,
    low: u8,
    high: u8,
    since: u64,
}

pub fn choose(ctx: &RuleContext, slot: u32) -> Option<u8> {
    let eligible = eligible(ctx, slot)?;
    let lane = eligible.probe.lane;
    if lane < eligible.low || lane > eligible.high {
        return mandatory(ctx, &eligible);
    }
    discretionary(ctx, &eligible)
}

pub fn is_safe(ctx: &RuleContext, slot: u32, target: u8) -> bool {
    let Place::Link { link, .. } = ctx.vehicles.place[slot as usize] else {
        return false;
    };
    let probe = Probe {
        slot,
        link,
        lane: target,
        s: ctx.vehicles.s[slot as usize],
    };
    assess(ctx, &probe).is_some()
}

fn eligible(ctx: &RuleContext, slot: u32) -> Option<Eligible> {
    let index = slot as usize;
    let Place::Link { link, lane } = ctx.vehicles.place[index] else {
        return None;
    };
    let since = ctx
        .tick
        .saturating_sub(ctx.vehicles.lane_change_tick[index]);
    if since < LC_ANIMATION_TICKS.min(LC_COOLDOWN_TICKS) || ctx.vehicles.committed[index] {
        return None;
    }
    let lanes = ctx.network.link_lanes(link);
    if lanes < 2 || ctx.occupancy.index_of(slot).is_none() {
        return None;
    }
    let (low, high) = allowed_range(ctx, slot, lanes)?;
    Some(Eligible {
        probe: Probe {
            slot,
            link,
            lane,
            s: ctx.vehicles.s[index],
        },
        low,
        high: high.min(lanes - 1),
        since,
    })
}

fn allowed_range(ctx: &RuleContext, slot: u32, lanes: u8) -> Option<(u8, u8)> {
    let ahead = current_ahead(ctx.network, ctx.vehicles, slot);
    if ahead.next().is_none() {
        return Some((0, lanes - 1));
    }
    ahead.from_lanes()
}

fn mandatory(ctx: &RuleContext, eligible: &Eligible) -> Option<u8> {
    if eligible.since < LC_ANIMATION_TICKS {
        return None;
    }
    let lane = eligible.probe.lane;
    let target = if lane < eligible.low {
        lane + 1
    } else {
        lane - 1
    };
    let probe = Probe {
        lane: target,
        ..eligible.probe
    };
    assess(ctx, &probe).map(|_| target)
}

fn discretionary(ctx: &RuleContext, eligible: &Eligible) -> Option<u8> {
    if !discretionary_due(ctx, eligible) {
        return None;
    }
    let lane = eligible.probe.lane;
    let right = lane.checked_sub(1).filter(|&l| l >= eligible.low);
    let left = lane.checked_add(1).filter(|&l| l <= eligible.high);
    if right.is_none() && left.is_none() {
        return None;
    }
    let current = Current::of(ctx, &eligible.probe);
    let score = |target: u8, bias: f64| incentive(ctx, &current, target, bias).map(|i| (target, i));
    let right = right.and_then(|l| score(l, -LC_KEEP_RIGHT_BIAS));
    let left = left.and_then(|l| score(l, LC_KEEP_RIGHT_BIAS));
    match (right, left) {
        (Some(r), Some(l)) if l.1 > r.1 => Some(l.0),
        (Some(r), _) => Some(r.0),
        (None, l) => l.map(|l| l.0),
    }
}

struct Current {
    probe: Probe,
    self_old: f64,
    follower_gain: f64,
}

impl Current {
    fn of(ctx: &RuleContext, probe: &Probe) -> Current {
        let leader = leader_on(ctx, probe);
        let follower_gain = follower_on(ctx, probe).map_or(0.0, |follower| {
            let before = follower.dist - ctx.vehicles.length(probe.slot);
            let v = ctx.vehicles.v[probe.slot as usize];
            accel(ctx, follower.slot, behind(follower.dist, leader))
                - accel(ctx, follower.slot, Some((before, v)))
        });
        Current {
            probe: *probe,
            self_old: accel(ctx, probe.slot, leader),
            follower_gain,
        }
    }
}

fn discretionary_due(ctx: &RuleContext, eligible: &Eligible) -> bool {
    let probe = &eligible.probe;
    let span_end = ctx.network.link_span(probe.link).1;
    eligible.since >= LC_COOLDOWN_TICKS
        && span_end - probe.s >= LC_MIN_DISTANCE_TO_END
        && (ctx.tick + u64::from(probe.slot)).is_multiple_of(LC_DISCRETIONARY_EVERY)
}

struct Assessment {
    self_new: f64,
    new_leader: Obstacle,
    follower: Option<(Follower, f64)>,
}

fn assess(ctx: &RuleContext, probe: &Probe) -> Option<Assessment> {
    let new_leader = leader_on(ctx, probe);
    if new_leader.is_some_and(|(gap, _)| gap < 0.0) {
        return None;
    }
    let self_new = accel(ctx, probe.slot, new_leader);
    if self_new < -LC_SAFE_DECEL {
        return None;
    }
    let follower = match follower_on(ctx, probe) {
        Some(follower) => Some((follower, follower_accel(ctx, probe, &follower)?)),
        None => None,
    };
    Some(Assessment {
        self_new,
        new_leader,
        follower,
    })
}

fn follower_accel(ctx: &RuleContext, probe: &Probe, follower: &Follower) -> Option<f64> {
    let gap = follower.dist - ctx.vehicles.length(probe.slot);
    if gap < 0.0 {
        return None;
    }
    if follower.linked && ctx.vehicles.committed[follower.slot as usize] {
        return None;
    }
    let a = accel(
        ctx,
        follower.slot,
        Some((gap, ctx.vehicles.v[probe.slot as usize])),
    );
    (a >= -LC_SAFE_DECEL).then_some(a)
}

fn incentive(ctx: &RuleContext, current: &Current, target: u8, bias: f64) -> Option<f64> {
    let probe = Probe {
        lane: target,
        ..current.probe
    };
    let new = assess(ctx, &probe)?;
    let self_gain = new.self_new - current.self_old;
    let new_follower_gain = new.follower.map_or(0.0, |(follower, after)| {
        after - accel(ctx, follower.slot, behind(follower.dist, new.new_leader))
    });
    let total = self_gain + LC_POLITENESS * (new_follower_gain + current.follower_gain);
    (total > LC_THRESHOLD + bias).then_some(total)
}

fn behind(dist: f64, leader: Obstacle) -> Obstacle {
    leader.map(|(gap, v)| (dist + gap, v))
}

fn accel(ctx: &RuleContext, slot: u32, obstacle: Obstacle) -> f64 {
    let index = slot as usize;
    let kind = ctx.vehicles.kind[index];
    let v = ctx.vehicles.v[index];
    let v0 = desired_speed(ctx, slot) * kind.speed_factor();
    match obstacle {
        Some((gap, leader_v)) => acceleration(v, v0, gap, v - leader_v, kind.accel()),
        None => acceleration(v, v0, f64::INFINITY, 0.0, kind.accel()),
    }
}

fn desired_speed(ctx: &RuleContext, slot: u32) -> f64 {
    let link = match ctx.vehicles.place[slot as usize] {
        Place::Link { link, .. } => Some(link),
        Place::Movement { .. } => ctx.vehicles.route_link(slot, 1),
    };
    link.map_or(0.0, |link| ctx.network.link_speed(link))
}
