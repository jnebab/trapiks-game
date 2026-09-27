use std::time::Instant;

use trapiks_sim_core::demand::Trip;
use trapiks_sim_core::network::LinkId;
use trapiks_sim_core::routing::{LinkCosts, RouteContext, RouteGraph, Router, dijkstra_cost};
use trapiks_sim_core::sim::Sim;

const WEIGHTS: [f64; 2] = [1.0, 2.0];

struct Outcome {
    micros: f64,
    expansions: f64,
    excess: f64,
    found: usize,
}

pub fn report(label: &str, sim: &Sim, costs: &LinkCosts, trips: &[Trip]) {
    let truth: Vec<Option<f64>> = trips
        .iter()
        .map(|trip| dijkstra_cost(sim.route_graph(), costs, trip.from, trip.to))
        .collect();
    println!(
        "[{label}] alt bound / true cost at start: {:.3}",
        bound_ratio(sim, costs, trips, &truth)
    );
    for weight in WEIGHTS {
        for alt in [false, true] {
            let ctx = context(sim, costs, weight, alt);
            let outcome = run(ctx, trips, &truth);
            print_outcome(label, weight, alt, &outcome);
        }
    }
}

fn print_outcome(label: &str, weight: f64, alt: bool, outcome: &Outcome) {
    let heuristic = if alt { "alt" } else { "euclid" };
    println!(
        "[{label}] weight {weight:.1} {heuristic}: {:.1} us, {:.0} expansions, excess {:.2} %, found {}",
        outcome.micros,
        outcome.expansions,
        outcome.excess * 100.0,
        outcome.found
    );
}

fn context<'a>(sim: &'a Sim, costs: &'a LinkCosts, weight: f64, alt: bool) -> RouteContext<'a> {
    RouteContext {
        graph: sim.route_graph(),
        costs,
        landmarks: alt.then(|| sim.landmarks()),
        network: sim.network(),
        heuristic_weight: weight,
    }
}

fn bound_ratio(sim: &Sim, costs: &LinkCosts, trips: &[Trip], truth: &[Option<f64>]) -> f64 {
    let landmarks = sim.landmarks();
    let mut sum = 0.0;
    let mut count = 0;
    for (trip, cost) in trips.iter().zip(truth) {
        let Some(cost) = cost.filter(|&c| c > 0.0) else {
            continue;
        };
        let target = landmarks.target(sim.network(), trip.to);
        sum += landmarks.estimate(sim.network(), costs, trip.from, &target) / cost;
        count += 1;
    }
    sum / f64::from(count.max(1))
}

fn run(ctx: RouteContext, trips: &[Trip], truth: &[Option<f64>]) -> Outcome {
    let mut router = Router::new();
    let mut route = Vec::new();
    let mut excess_sum = 0.0;
    let mut found = 0;
    let mut micros = 0.0;
    for (trip, cost) in trips.iter().zip(truth) {
        let started = Instant::now();
        let ok = router.route_into(ctx, trip.from, trip.to, &mut route);
        micros += started.elapsed().as_secs_f64() * 1e6;
        if let (true, Some(cost)) = (ok, cost.filter(|&c| c > 0.0)) {
            excess_sum += path_cost(ctx.graph, ctx.costs, &route) / cost - 1.0;
            found += 1;
        }
    }
    let n = trips.len().max(1) as f64;
    Outcome {
        micros: micros / n,
        expansions: router.stats().expanded as f64 / n,
        excess: excess_sum / found.max(1) as f64,
        found,
    }
}

fn path_cost(graph: &RouteGraph, costs: &LinkCosts, route: &[LinkId]) -> f64 {
    route
        .windows(2)
        .map(|pair| step_cost(graph, costs, pair[0], pair[1]))
        .sum()
}

fn step_cost(graph: &RouteGraph, costs: &LinkCosts, from: LinkId, to: LinkId) -> f64 {
    let penalty = graph
        .successors(from)
        .iter()
        .filter(|s| s.to == to)
        .map(|s| f64::from(s.penalty))
        .fold(f64::INFINITY, f64::min);
    costs.travel_time(to) + penalty
}
