mod support;

use support::{apply_now, check_invariants, slot_of};
use trapiks_sim_core::edit::{EditCommand, Outcome};
use trapiks_sim_core::fixtures::{corridor, four_way};
use trapiks_sim_core::network::{Network, TurnKind, road_of};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const CORRIDOR_ROUTE: [u32; 3] = [0, 2, 4];

fn ids_on_road(sim: &Sim, road: u32) -> Vec<u32> {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter(|&slot| match vehicles.place[slot as usize] {
            Place::Link { link, .. } => road_of(link) == road,
            Place::Movement { .. } => false,
        })
        .map(|slot| vehicles.id[slot as usize])
        .collect()
}

fn fill_corridor(sim: &mut Sim) {
    let mut next_spawn = 0;
    while sim.tick() < 3_000 {
        if ids_on_road(sim, 1).len() >= 2 && ids_on_road(sim, 0).len() >= 2 {
            return;
        }
        if sim.tick() >= next_spawn && sim.spawn(&CORRIDOR_ROUTE).is_ok() {
            next_spawn = sim.tick() + 25;
        }
        sim.step();
    }
    panic!("corridor never filled");
}

fn drain_flagged(sim: &mut Sim) {
    for _ in 0..10 {
        if sim.flagged_slots().count() == 0 {
            return;
        }
        sim.step();
    }
}

#[test]
fn delete_road_vehicles() {
    let mut sim = Sim::new(&corridor(), 1);
    fill_corridor(&mut sim);
    let on_deleted = ids_on_road(&sim, 1);
    let behind = ids_on_road(&sim, 0);
    let stranded_before = sim.stranded();
    let outcome = apply_now(&mut sim, EditCommand::DeleteRoad { road: 1 });
    assert!(matches!(outcome, Outcome::Ok(_)), "{outcome:?}");
    drain_flagged(&mut sim);
    for &id in on_deleted.iter().chain(&behind) {
        assert!(slot_of(&sim, id).is_none(), "vehicle {id} survived");
    }
    let counts = sim.reroute_counts();
    assert!(counts.marked >= behind.len() as u64, "{counts:?}");
    assert!(sim.stranded() - stranded_before >= (on_deleted.len() + behind.len()) as u64);
    assert_eq!(sim.flagged_slots().count(), 0);
    for _ in 0..200 {
        sim.step();
        check_invariants(&sim);
    }
}

fn left_route() -> Option<[u32; 2]> {
    let mut network = Network::from_map(&four_way(2, 200.0));
    network
        .ensure_junction(0)
        .movements
        .iter()
        .find(|m| m.from_link == 0 && m.kind == TurnKind::Left)
        .map(|m| [m.from_link, m.to_link])
}

fn queue_left_turners(sim: &mut Sim, route: [u32; 2]) {
    let mut next_spawn = 0;
    while sim.tick() < 280 {
        if sim.tick() >= next_spawn && sim.spawn(&route).is_ok() {
            next_spawn = sim.tick() + 20;
        }
        sim.step();
    }
}

fn lane_one_count(sim: &Sim) -> usize {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter(|&slot| {
            matches!(
                vehicles.place[slot as usize],
                Place::Link { link: 0, lane: 1 }
            )
        })
        .count()
}

#[test]
fn lanes_edit_clamps() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let route = left_route().expect("left turn");
    queue_left_turners(&mut sim, route);
    assert!(lane_one_count(&sim) >= 2);
    let outcome = apply_now(
        &mut sim,
        EditCommand::SetLanes {
            road: 0,
            forward: 1,
            backward: 2,
        },
    );
    assert!(matches!(outcome, Outcome::Ok(_)), "{outcome:?}");
    assert_eq!(lane_one_count(&sim), 0);
    let vehicles = sim.vehicles();
    for slot in vehicles.live_slots() {
        let index = slot as usize;
        assert!(!vehicles.s[index].is_nan() && !vehicles.v[index].is_nan());
    }
    for _ in 0..50 {
        sim.step();
    }
    check_invariants(&sim);
}

fn movement_pairs(sim: &Sim) -> Vec<(u32, u32, u32)> {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter_map(|slot| match vehicles.place[slot as usize] {
            Place::Movement { node, movement, .. } => {
                let m = &sim.network().junction(node)?.movements[usize::from(movement)];
                Some((vehicles.id[slot as usize], m.from_link, m.to_link))
            }
            Place::Link { .. } => None,
        })
        .collect()
}

#[test]
fn movement_remap_keeps_pairs() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let mut next = 0;
    while movement_pairs(&sim).is_empty() && sim.tick() < 3_000 {
        if sim.tick() >= next && sim.spawn(&[0, 5]).is_ok() {
            next = sim.tick() + 15;
        }
        sim.step();
    }
    let before = movement_pairs(&sim);
    assert!(!before.is_empty());
    let outcome = apply_now(&mut sim, EditCommand::DeleteRoad { road: 1 });
    assert!(matches!(outcome, Outcome::Ok(_)), "{outcome:?}");
    let after = movement_pairs(&sim);
    for entry in &before {
        let survived = after.iter().any(|a| a.0 == entry.0);
        let still_inside = survived && after.contains(entry);
        let moved_on = slot_of(&sim, entry.0).is_some() && !survived;
        assert!(still_inside || moved_on, "{entry:?} {after:?}");
    }
    check_invariants(&sim);
}
