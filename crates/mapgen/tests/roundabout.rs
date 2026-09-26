mod support;

use support::Fixture;
use trapiks_sim_core::consts::MINOR_GAP;
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::network::{LinkId, Network, road_of};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;
use trapiks_sim_core::vehicle::approach::{approach, approach_time};
use trapiks_sim_core::vehicle::entry::no_priority_approach;

const RING_EVERY: u64 = 25;
const RING_UNTIL: u64 = 500;
const MAX_TICKS: u64 = 1_500;

fn load() -> MapData {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/roundabout.json"
    );
    let Ok((osm, _)) = trapiks_mapgen::input::load(&[path.to_string()]) else {
        panic!("load failed");
    };
    Fixture { osm }.build()
}

fn ring_roads(map: &MapData) -> Vec<usize> {
    (0..map.road_count())
        .filter(|&r| map.roads.roundabout[r])
        .collect()
}

#[test]
fn osm_roundabout_is_flagged_and_unmerged() {
    let map = load();
    assert_eq!(map.road_count(), 8);
    let ring = ring_roads(&map);
    assert_eq!(ring.len(), 4);
    for &road in &ring {
        assert_eq!(map.roads.lanes_backward[road], 0);
    }
    let entries: Vec<u32> = ring.iter().map(|&r| map.roads.from[r]).collect();
    for node in 0..map.node_count() as u32 {
        let expected = if entries.contains(&node) {
            Control::Yield
        } else {
            Control::Priority
        };
        assert_eq!(map.nodes.control[node as usize], expected);
    }
}

struct Layout {
    arm_in: LinkId,
    ring_prev: LinkId,
    ring_in: LinkId,
    ring_out: LinkId,
    ring_next: LinkId,
}

fn ring_link_into(network: &Network, node: u32) -> LinkId {
    network
        .incoming(node)
        .find(|&link| network.roads.is_roundabout(road_of(link)))
        .unwrap_or_default()
}

fn ring_link_out_of(network: &Network, node: u32) -> LinkId {
    network
        .outgoing(node)
        .find(|&link| network.roads.is_roundabout(road_of(link)))
        .unwrap_or_default()
}

fn layout(network: &Network, entry: u32) -> Layout {
    let arm_in = network
        .incoming(entry)
        .find(|&link| !network.roads.is_roundabout(road_of(link)))
        .unwrap_or_default();
    let ring_in = ring_link_into(network, entry);
    let ring_out = ring_link_out_of(network, entry);
    Layout {
        arm_in,
        ring_prev: ring_link_into(network, network.link_from(ring_in)),
        ring_in,
        ring_out,
        ring_next: ring_link_out_of(network, network.link_to(ring_out)),
    }
}

fn close_ring_vehicle(sim: &Sim, entrant: u32, layout: &Layout) -> bool {
    let ctx = sim.rule_context();
    let Some(a) = approach(&ctx, entrant) else {
        return false;
    };
    let vehicles = sim.vehicles();
    vehicles.live_slots().any(|slot| {
        let on_ring_in = matches!(
            vehicles.place[slot as usize],
            Place::Link { link, .. } if link == layout.ring_in
        );
        let Some(b) = approach(&ctx, slot).filter(|_| on_ring_in) else {
            return false;
        };
        a.junction.conflicts[usize::from(a.index)]
            .iter()
            .filter(|c| c.other == b.index)
            .any(|c| approach_time(&ctx, &b, b.span_end - b.s + c.s_other) < MINOR_GAP)
    })
}

#[test]
fn osm_roundabout_entry_yields_to_ring() {
    let map = load();
    let mut sim = Sim::new(&map, 1);
    let entry = sim.network().roads.from[ring_roads(&map)[0]];
    let layout = layout(sim.network(), entry);
    let entrant_id = sim
        .spawn(&[layout.arm_in, layout.ring_out, layout.ring_next])
        .expect("entrant");
    let mut yielded = false;
    let mut committed = false;
    while sim.tick() < MAX_TICKS && !committed {
        if sim.tick() < RING_UNTIL && sim.tick().is_multiple_of(RING_EVERY) {
            let route = [layout.ring_prev, layout.ring_in, layout.ring_out];
            let _ = sim.spawn(&route);
        }
        sim.step();
        let entrant = sim
            .vehicles()
            .live_slots()
            .find(|&slot| sim.vehicles().id[slot as usize] == entrant_id)
            .expect("entrant alive");
        let index = entrant as usize;
        if sim.vehicles().committed[index] {
            committed = true;
            assert!(!close_ring_vehicle(&sim, entrant, &layout));
            continue;
        }
        let waiting = approach(&sim.rule_context(), entrant).is_some()
            && !no_priority_approach(&sim.rule_context(), entrant);
        yielded |= waiting && close_ring_vehicle(&sim, entrant, &layout);
    }
    assert!(yielded);
    assert!(committed);
}
