use std::io::Read;
use std::time::Instant;

use flate2::read::GzDecoder;
use trapiks_sim_core::consts::LANDMARK_COUNT;
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::routing::{Landmarks, LinkCosts, RouteGraph};
use trapiks_sim_core::sim::Sim;

const TRIPS: usize = 2_000;

struct Timings {
    micros: Vec<f64>,
    links: usize,
    found: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: route_bench <map.bin.gz>")?;
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut bytes)?;
    let map = from_bytes(&bytes)?;
    let landmark_ms = landmark_build_ms(&map);
    let mut sim = Sim::with_capacity(&map, 1, 16);
    let timings = run_trips(&mut sim);
    report(&sim, landmark_ms, timings);
    Ok(())
}

fn landmark_build_ms(map: &MapData) -> f64 {
    let network = Network::from_map(map);
    let graph = RouteGraph::build(&network);
    let costs = LinkCosts::build(&network);
    let started = Instant::now();
    let landmarks = Landmarks::build(&network, &graph, &costs);
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    assert!(network.link_count() < LANDMARK_COUNT || landmarks.count() == LANDMARK_COUNT);
    println!("landmarks: {}", landmarks.count());
    millis
}

fn random_active(network: &Network, rng: &mut Pcg32) -> LinkId {
    loop {
        let link = rng.below(network.link_count() as u32);
        if network.is_link_active(link) {
            return link;
        }
    }
}

fn run_trips(sim: &mut Sim) -> Timings {
    let mut rng = Pcg32::new(7, 1);
    let mut timings = Timings {
        micros: Vec::with_capacity(TRIPS),
        links: 0,
        found: 0,
    };
    for _ in 0..TRIPS {
        let from = random_active(sim.network(), &mut rng);
        let to = random_active(sim.network(), &mut rng);
        let started = Instant::now();
        let route = sim.route_with_weight(from, to, 1.0);
        timings.micros.push(started.elapsed().as_secs_f64() * 1e6);
        if let Some(route) = route {
            timings.links += route.len();
            timings.found += 1;
        }
    }
    timings
}

fn report(sim: &Sim, landmark_ms: f64, mut timings: Timings) {
    timings.micros.sort_by(f64::total_cmp);
    let count = timings.micros.len().max(1);
    let mean = timings.micros.iter().sum::<f64>() / count as f64;
    let p95 = timings.micros[(count * 95 / 100).min(count - 1)];
    let stats = sim.route_stats();
    let queries = stats.queries.max(1);
    println!("links: {}", sim.network().link_count());
    println!("landmark build ms: {landmark_ms:.1}");
    println!("routes found: {} / {}", timings.found, timings.micros.len());
    println!("mean route us: {mean:.1}");
    println!("p95 route us: {p95:.1}");
    println!(
        "mean expansions: {:.1}",
        stats.expanded as f64 / queries as f64
    );
    println!(
        "mean route links: {:.1}",
        timings.links as f64 / timings.found.max(1) as f64
    );
}
