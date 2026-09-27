mod support;

use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

use flate2::read::GzDecoder;
use support::Must;
use trapiks_sim_core::consts::CITY_TRIP_BAND;
use trapiks_sim_core::demand::{Trip, draw_trip};
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::routing::{RouteGraph, largest_component};
use trapiks_sim_core::sim::Sim;

const TRIPS: usize = 2_000;
const DRAW_ATTEMPTS: usize = 200_000;
const MIN_SCC_SHARE: f64 = 0.95;
const MIN_ROUTE_SHARE: f64 = 0.97;

fn real_map() -> Option<MapData> {
    if std::env::var("TRAPIKS_REAL_MAP").ok().as_deref() != Some("1") {
        println!("skipped: set TRAPIKS_REAL_MAP=1");
        return None;
    }
    let path = std::env::var("TRAPIKS_MAP_PATH").map_or_else(|_| default_path(), PathBuf::from);
    println!("map: {}", path.display());
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(&path).must("open map"))
        .read_to_end(&mut bytes)
        .must("gunzip map");
    Some(from_bytes(&bytes).must("decode map"))
}

fn default_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/maps/metro-manila.bin.gz")
}

#[test]
#[ignore]
fn every_junction_builds() {
    let Some(map) = real_map() else { return };
    let mut network = Network::from_map(&map);
    let started = Instant::now();
    network.build_all_junctions();
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    let built = (0..network.nodes.count() as u32)
        .filter(|&node| network.junction(node).is_some())
        .count();
    println!("junctions built: {built}");
    println!("junction build ms: {millis:.1}");
}

#[test]
#[ignore]
fn largest_component_covers_network() {
    let Some(map) = real_map() else { return };
    let network = Network::from_map(&map);
    let graph = RouteGraph::build(&network);
    let active = (0..network.link_count() as LinkId)
        .filter(|&link| network.is_link_active(link))
        .count();
    let largest = largest_component(&graph, |link| network.is_link_active(link));
    let share = largest as f64 / active.max(1) as f64;
    println!("active links: {active}");
    println!("largest scc: {largest} ({:.2} %)", share * 100.0);
    assert!(share >= MIN_SCC_SHARE, "scc share {share:.4}");
}

fn draw_trips(sim: &Sim) -> Vec<Trip> {
    let mut rng = Pcg32::new(16, 1);
    let mut trips = Vec::with_capacity(TRIPS);
    for _ in 0..DRAW_ATTEMPTS {
        if trips.len() == TRIPS {
            break;
        }
        let drawn = draw_trip(
            sim.demand_tables(),
            sim.network(),
            CITY_TRIP_BAND,
            0,
            &mut rng,
        );
        trips.extend(drawn);
    }
    trips
}

#[test]
#[ignore]
fn random_trips_find_routes() {
    let Some(map) = real_map() else { return };
    let mut sim = Sim::with_capacity(&map, 16, 16);
    let trips = draw_trips(&sim);
    assert_eq!(trips.len(), TRIPS, "could not draw enough 1-12 km trips");
    let started = Instant::now();
    let found = trips
        .iter()
        .filter(|trip| sim.route_with_weight(trip.from, trip.to, 1.0).is_some())
        .count();
    let mean_us = started.elapsed().as_secs_f64() * 1e6 / TRIPS as f64;
    let share = found as f64 / TRIPS as f64;
    println!("routes found: {found} / {TRIPS} ({:.2} %)", share * 100.0);
    println!("mean a* us: {mean_us:.1}");
    assert!(share >= MIN_ROUTE_SHARE, "route share {share:.4}");
}
