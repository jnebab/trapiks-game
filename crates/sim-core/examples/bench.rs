use std::io::Read;
use std::time::Instant;

use flate2::read::GzDecoder;
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::EditCommand;
use trapiks_sim_core::map::RoadClass;
use trapiks_sim_core::map::from_bytes;
use trapiks_sim_core::network::road_of;
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::routing::RouteStats;
use trapiks_sim_core::sim::{RerouteCounts, Sim};
use trapiks_sim_core::vehicle::Place;

const ROUTE_SAMPLES: usize = 2_000;
const DELETE_WINDOW: usize = 600;

struct Args {
    path: String,
    mode: SimMode,
    vph: f64,
    warmup: u64,
    ticks: u64,
    delete_at: Option<u64>,
}

#[derive(Default)]
struct Run {
    step_ms: Vec<f64>,
    active_sum: f64,
    active_max: u32,
    speed_sum: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(&args.path)?).read_to_end(&mut bytes)?;
    let map = from_bytes(&bytes)?;
    let config = SimConfig {
        seed: 1,
        mode: args.mode,
        vehicles_per_hour: args.vph,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    for _ in 0..args.warmup {
        sim.step();
    }
    sim.reset_stats();
    let before = sim.route_stats();
    let run = measure(&mut sim, args.ticks, args.delete_at);
    let routing = sim.route_stats();
    let state_hash = sim.state_hash();
    let route_us = mean_route_us(&mut sim);
    report(&sim, &run, delta(before, routing), route_us, args.ticks);
    println!("state_hash: {state_hash:#018x}");
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut raw = std::env::args().skip(1);
    let usage = "usage: bench <map.bin.gz> [--region x,y,r] [--vph N] [--warmup N] [--ticks N] [--delete-road-at TICK]";
    let mut args = Args {
        path: raw.next().ok_or(usage)?,
        mode: SimMode::City,
        vph: 0.0,
        warmup: 0,
        ticks: 1_000,
        delete_at: None,
    };
    while let Some(flag) = raw.next() {
        let value = raw.next().ok_or(usage)?;
        match flag.as_str() {
            "--region" => args.mode = parse_region(&value)?,
            "--vph" => args.vph = value.parse()?,
            "--warmup" => args.warmup = value.parse()?,
            "--ticks" => args.ticks = value.parse()?,
            "--delete-road-at" => args.delete_at = Some(value.parse()?),
            _ => return Err(usage.into()),
        }
    }
    Ok(args)
}

fn parse_region(value: &str) -> Result<SimMode, Box<dyn std::error::Error>> {
    let parts: Vec<f64> = value.split(',').map(str::parse).collect::<Result<_, _>>()?;
    let [center_x, center_y, radius] = parts[..] else {
        return Err("--region takes x,y,r".into());
    };
    Ok(SimMode::Region {
        center_x,
        center_y,
        radius,
    })
}

fn measure(sim: &mut Sim, ticks: u64, delete_at: Option<u64>) -> Run {
    let mut run = Run::default();
    let mut deletion = None;
    for _ in 0..ticks {
        let deleting = delete_at == Some(sim.tick());
        let before = deleting.then(|| delete_busiest_primary(sim));
        let started = Instant::now();
        sim.step();
        run.step_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        if let Some(before) = before {
            deletion = Some((first_step_counts(sim, &before), run.step_ms.len() - 1));
        }
        let stats = sim.stats();
        run.active_sum += f64::from(stats.active);
        run.active_max = run.active_max.max(stats.active);
        run.speed_sum += stats.mean_speed;
    }
    if let Some((counts, start)) = deletion {
        report_deletion(&counts, &run.step_ms[start..]);
    }
    run
}

struct Deletion {
    road: u32,
    vehicles: usize,
    tick: u64,
    stranded: u64,
    reroutes: RerouteCounts,
}

fn busiest_primary(sim: &Sim) -> Option<(u32, usize)> {
    let network = sim.network();
    let vehicles = sim.vehicles();
    let mut counts = vec![0usize; network.roads.count()];
    for slot in vehicles.live_slots() {
        if let Place::Link { link, .. } = vehicles.place[slot as usize] {
            counts[road_of(link) as usize] += 1;
        }
    }
    let eligible = |road: usize| {
        network.roads.class[road] == RoadClass::Primary && network.roads.is_live(road as u32)
    };
    counts
        .iter()
        .enumerate()
        .filter(|&(road, _)| eligible(road))
        .max_by_key(|&(road, &count)| (count, std::cmp::Reverse(road)))
        .map(|(road, &count)| (road as u32, count))
}

fn delete_busiest_primary(sim: &mut Sim) -> Deletion {
    let (road, vehicles) = busiest_primary(sim).unwrap_or((0, 0));
    sim.enqueue(EditCommand::DeleteRoad { road });
    Deletion {
        road,
        vehicles,
        tick: sim.tick(),
        stranded: sim.stranded(),
        reroutes: sim.reroute_counts(),
    }
}

fn first_step_counts(sim: &Sim, before: &Deletion) -> Deletion {
    let after = sim.reroute_counts();
    Deletion {
        stranded: sim.stranded() - before.stranded,
        reroutes: RerouteCounts {
            immediate: after.immediate - before.reroutes.immediate,
            flagged: after.flagged - before.reroutes.flagged,
            marked: after.marked - before.reroutes.marked,
        },
        ..*before
    }
}

fn report_deletion(counts: &Deletion, steps: &[f64]) {
    let window = &steps[..steps.len().min(DELETE_WINDOW)];
    let max = window.iter().copied().fold(0.0, f64::max);
    println!(
        "delete: road {} ({} vehicles on it) at tick {}",
        counts.road, counts.vehicles, counts.tick
    );
    println!(
        "delete step: stranded {} immediate reroutes {} flagged {} flagged rerouted {}",
        counts.stranded, counts.reroutes.immediate, counts.reroutes.marked, counts.reroutes.flagged
    );
    println!(
        "delete step ms: first {:.2} max over {} ticks {max:.2}",
        window.first().copied().unwrap_or(0.0),
        window.len()
    );
}

fn delta(before: RouteStats, after: RouteStats) -> RouteStats {
    RouteStats {
        queries: after.queries - before.queries,
        expanded: after.expanded - before.expanded,
    }
}

fn mean_route_us(sim: &mut Sim) -> f64 {
    let mut rng = Pcg32::new(7, 1);
    let mut total = 0.0;
    for _ in 0..ROUTE_SAMPLES {
        let tables = sim.demand_tables();
        let (Some(from), Some(to)) = (
            tables.origins.sample(&mut rng),
            tables.destinations.sample(&mut rng),
        ) else {
            return 0.0;
        };
        let started = Instant::now();
        let _ = sim.route(from, to);
        total += started.elapsed().as_secs_f64() * 1e6;
    }
    total / ROUTE_SAMPLES as f64
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    let index = ((sorted.len() as f64 * fraction) as usize).min(sorted.len().saturating_sub(1));
    sorted.get(index).copied().unwrap_or(0.0)
}

fn report(sim: &Sim, run: &Run, routing: RouteStats, route_us: f64, ticks: u64) {
    let mut sorted = run.step_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let ticks = ticks.max(1) as f64;
    let stats = sim.stats();
    let routes_per_step = routing.queries as f64 / ticks;
    let junctions = (0..sim.network().nodes.count() as u32)
        .filter(|&node| sim.network().junction(node).is_some())
        .count();
    println!(
        "step ms: median {:.2} p95 {:.2} max {:.2}",
        percentile(&sorted, 0.5),
        percentile(&sorted, 0.95),
        sorted.last().copied().unwrap_or(0.0)
    );
    println!(
        "active: mean {:.0} max {}",
        run.active_sum / ticks,
        run.active_max
    );
    println!("mean speed: {:.2} m/s", run.speed_sum / ticks);
    println!(
        "created {} spawned {} arrivals {} unserved {} stranded {}",
        stats.created, stats.spawned, stats.arrivals, stats.unserved, stats.stranded
    );
    println!("queued trips: {}", sim.queued_trips());
    println!("routes per step: {routes_per_step:.2}");
    println!(
        "mean expansions per route: {:.1}",
        routing.expanded as f64 / routing.queries.max(1) as f64
    );
    println!("mean route us: {route_us:.1}");
    println!(
        "route work per step ms: {:.3}",
        routes_per_step * route_us / 1000.0
    );
    println!("junctions built: {junctions}");
    println!("landmarks: {}", sim.landmark_count());
}
