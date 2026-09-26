mod support;

use support::{check_invariants, slot_of};
use trapiks_sim_core::consts::STOPPED_SPEED;
use trapiks_sim_core::fixtures::{MapBuilder, RoadSpec};
use trapiks_sim_core::map::{Control, MapData, RoadClass};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const LEFT: [u32; 3] = [4, 10, 3];
const BLOCKER: [u32; 2] = [10, 3];
const LEFT_TICK: u64 = 400;
const FEED_TICK: u64 = 680;
const QUEUED_BLOCKERS: u32 = 3;
const END_TICK: u64 = 1100;
const CENTRE: u32 = 0;
const THROUGH: [u32; 2] = [6, 5];

fn stalled_box() -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    let west = b.node(34.0, 0.0);
    b.control(centre, Control::Signal);
    b.control(west, Control::Signal);
    let ends = [
        (34.0, 200.0),
        (34.0, -200.0),
        (0.0, -120.0),
        (0.0, 420.0),
        (-200.0, 0.0),
    ]
    .map(|(x, y)| b.node(x, y));
    let street = RoadSpec::new(RoadClass::Primary, 1, 1);
    b.road(ends[0], west, street.clone());
    b.road(ends[1], west, street.clone());
    b.road(ends[2], centre, street.clone());
    b.road(ends[3], centre, street.clone().speed(120));
    b.road(ends[4], centre, street.clone());
    b.road(centre, west, street);
    b.build()
}

fn movement_s(sim: &Sim, id: u32) -> Option<(u32, u16, f64)> {
    let slot = slot_of(sim, id)? as usize;
    match sim.vehicles().place[slot] {
        Place::Movement { node, movement, .. } => Some((node, movement, sim.vehicles().s[slot])),
        Place::Link { .. } => None,
    }
}

fn should_reblock(sim: &Sim, left: u32) -> bool {
    let Some(slot) = slot_of(sim, left) else {
        return false;
    };
    let index = slot as usize;
    let vehicles = sim.vehicles();
    match vehicles.place[index] {
        Place::Movement { node, .. } => node == CENTRE,
        Place::Link { .. } => vehicles.committed[index],
    }
}

fn committed(sim: &Sim, id: Option<u32>) -> bool {
    id.and_then(|id| slot_of(sim, id))
        .is_some_and(|slot| sim.vehicles().committed[slot as usize])
}

#[derive(Default)]
struct Scenario {
    left: Option<u32>,
    through: Option<u32>,
    queued: u32,
    reblocked: bool,
    through_first: bool,
    held: bool,
}

impl Scenario {
    fn spawn(&mut self, sim: &mut Sim, through_tick: u64) {
        if sim.tick() == LEFT_TICK {
            self.left = sim.spawn(&LEFT).ok();
        }
        if sim.tick() == through_tick {
            self.through = sim.spawn(&THROUGH).ok();
        }
        if sim.tick() >= FEED_TICK && self.queued < QUEUED_BLOCKERS && sim.spawn(&BLOCKER).is_ok() {
            self.queued += 1;
        }
        let Some(left) = self.left else {
            return;
        };
        if !self.reblocked && should_reblock(sim, left) {
            self.through_first = committed(sim, self.through);
            self.reblocked = sim.spawn(&BLOCKER).is_ok();
        }
    }

    fn observe(&mut self, sim: &Sim) {
        let Some(left) = self.left else {
            return;
        };
        self.held |= stopped(sim, left) && through_held(sim, left, self.through);
    }
}

fn run(through_tick: u64) -> bool {
    let mut sim = Sim::new(&stalled_box(), 1);
    let mut scenario = Scenario::default();
    while sim.tick() < END_TICK {
        scenario.spawn(&mut sim, through_tick);
        sim.step();
        check_invariants(&sim);
        scenario.observe(&sim);
    }
    scenario.through_first && scenario.held
}

fn stopped(sim: &Sim, id: u32) -> bool {
    slot_of(sim, id).is_some_and(|slot| sim.vehicles().v[slot as usize] < STOPPED_SPEED)
}

fn crossing_point(sim: &Sim, left: (u32, u16, f64), through_movement: u16) -> Option<f64> {
    let junction = sim.network().junction(left.0)?;
    junction.conflicts[usize::from(through_movement)]
        .iter()
        .find(|c| c.other == left.1 && !c.merge)
        .map(|c| c.s_self)
}

#[test]
fn box_stalled_left_turner_holds_committed_through() {
    assert!((560..720).any(run));
}

fn through_held(sim: &Sim, left: u32, through: Option<u32>) -> bool {
    let (Some(l), Some(t)) = (movement_s(sim, left), through) else {
        return false;
    };
    let Some(slot) = slot_of(sim, t) else {
        return false;
    };
    let vehicles = sim.vehicles();
    let index = slot as usize;
    let Some(junction) = sim.network().junction(l.0) else {
        return false;
    };
    let Some(through_movement) = junction.movement_index(THROUGH[0], THROUGH[1]) else {
        return false;
    };
    let Some(point) = crossing_point(sim, l, through_movement as u16) else {
        return false;
    };
    let s = match vehicles.place[index] {
        Place::Movement { .. } => vehicles.s[index],
        Place::Link { link, .. } => vehicles.s[index] - sim.network().link_span(link).1,
    };
    let committed =
        vehicles.committed[index] || matches!(vehicles.place[index], Place::Movement { .. });
    committed && vehicles.v[index] < STOPPED_SPEED && s < point
}
