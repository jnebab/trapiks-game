mod eval;
mod placement;

use std::io::Read;

use flate2::read::GzDecoder;
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::consts::CITY_TRIP_BAND;
use trapiks_sim_core::demand::{Trip, draw_trip};
use trapiks_sim_core::map::from_bytes;
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::routing::LinkCosts;
use trapiks_sim_core::sim::Sim;

const TRIPS: usize = 2_000;
const DRAW_ATTEMPTS: usize = 200_000;

struct Args {
    path: String,
    vph: f64,
    warmup: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(&args.path)?).read_to_end(&mut bytes)?;
    let map = from_bytes(&bytes)?;
    let config = SimConfig {
        seed: 1,
        mode: SimMode::City,
        vehicles_per_hour: args.vph,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    placement::report(&sim);
    let trips = draw_trips(&sim);
    println!("trips: {}", trips.len());
    let free = LinkCosts::build(sim.network());
    eval::report("free-flow", &sim, &free, &trips);
    for _ in 0..args.warmup {
        sim.step();
    }
    println!("active after warm-up: {}", sim.vehicle_count());
    let warm = sim.link_costs().clone();
    eval::report("warmed", &sim, &warm, &trips);
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let usage = "usage: route_diag <map.bin.gz> [--vph N] [--warmup N]";
    let mut raw = std::env::args().skip(1);
    let mut args = Args {
        path: raw.next().ok_or(usage)?,
        vph: 0.0,
        warmup: 0,
    };
    while let Some(flag) = raw.next() {
        let value = raw.next().ok_or(usage)?;
        match flag.as_str() {
            "--vph" => args.vph = value.parse()?,
            "--warmup" => args.warmup = value.parse()?,
            _ => return Err(usage.into()),
        }
    }
    Ok(args)
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
