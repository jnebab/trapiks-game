mod args;
mod baseline;
mod expected;
mod probe;
mod search;
mod site;
mod variants;

use std::error::Error;
use std::io::Read;

use flate2::read::GzDecoder;
use trapiks_sim_core::challenge::{Center, Challenge};
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::network::Network;

use args::Args;
use baseline::{print_result, run_baseline};
use expected::{SEARCH_RADIUS_M, expected_names, names_match};
use probe::Context;
use search::run_probes;
use site::{CLUSTER_RADIUS_M, Locator, Site};

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse(std::env::args().skip(1))?;
    let map = load_map(&args.map)?;
    let mut challenges: Vec<Challenge> =
        serde_json::from_str(&std::fs::read_to_string(&args.challenges)?)?;
    let network = Network::from_map(&map);
    let locator = Locator::new(&map, &network);
    for challenge in challenges.iter_mut().filter(|c| selected(&args, c)) {
        override_demand(&args, challenge);
        check_challenge(&locator, challenge, &args)?;
    }
    write_fixed(&args, &challenges)
}

fn load_map(path: &str) -> Result<MapData, Box<dyn Error>> {
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut bytes)?;
    Ok(from_bytes(&bytes)?)
}

fn selected(args: &Args, challenge: &Challenge) -> bool {
    args.only.as_ref().is_none_or(|id| *id == challenge.id)
}

fn override_demand(args: &Args, challenge: &mut Challenge) {
    if let Some(vph) = args.vph {
        challenge.vehicles_per_hour = vph;
    }
    if let Some(radius) = args.radius {
        challenge.radius_m = radius;
    }
}

struct Located {
    anchor: u32,
    move_to: Option<Site>,
}

fn check_challenge(
    locator: &Locator,
    challenge: &mut Challenge,
    args: &Args,
) -> Result<(), Box<dyn Error>> {
    println!(
        "challenge {} ({}) radius={} vph={} budget={}",
        challenge.id,
        challenge.name,
        challenge.radius_m,
        challenge.vehicles_per_hour,
        challenge.budget
    );
    let located = locate(locator, challenge)?;
    if args.fix.is_some()
        && let Some(site) = &located.move_to
    {
        challenge.center = Center::LatLon {
            lat: site.lat,
            lon: site.lon,
        };
        println!("  fixed centre -> {:.6}, {:.6}", site.lat, site.lon);
    }
    let node = locator.busiest_in_cluster(located.anchor);
    println!(
        "  probe node: {node} (most incident lanes within {CLUSTER_RADIUS_M} m of node {})",
        located.anchor
    );
    if args.sites_only {
        return Ok(());
    }
    run_checks(locator.map, challenge, node, args.probes)
}

fn locate(locator: &Locator, challenge: &Challenge) -> Result<Located, Box<dyn Error>> {
    let (x, y) = challenge.center_xy(locator.map);
    let center = Vec2 { x, y };
    let expected = expected_names(&challenge.id);
    let nearest = locator
        .nearest(center)
        .ok_or("no junction with active degree >= 3")?;
    print_site("nearest junction", &nearest);
    let cluster = locator.cluster_names(center);
    let on_target = names_match(expected, &cluster);
    println!("  names within {CLUSTER_RADIUS_M} m: {cluster:?}");
    println!(
        "  centre check: {}",
        if on_target { "PASS" } else { "FAIL" }
    );
    if on_target {
        return Ok(Located {
            anchor: nearest.node,
            move_to: None,
        });
    }
    let best = find_candidate(locator, center, expected);
    Ok(Located {
        anchor: best.as_ref().map_or(nearest.node, |site| site.node),
        move_to: best,
    })
}

fn find_candidate(locator: &Locator, center: Vec2, expected: &[&[&str]]) -> Option<Site> {
    let radius = SEARCH_RADIUS_M;
    let best = locator.best_match(center, radius, expected);
    match &best {
        Some(site) => print_site("best named candidate", site),
        None => {
            println!("  best named candidate: none within {radius} m matching {expected:?}");
            let names = locator.road_names(&locator.within(center, radius));
            println!("  names within {radius} m: {names:?}");
        }
    }
    best
}

fn print_site(label: &str, site: &Site) {
    println!(
        "  {label}: node {} at {:.1} m, lat/lon {:.6}, {:.6}, roads {:?}",
        site.node, site.distance, site.lat, site.lon, site.names
    );
}

fn run_checks(
    map: &MapData,
    challenge: &Challenge,
    node: u32,
    probes: bool,
) -> Result<(), Box<dyn Error>> {
    let baseline = run_baseline(map, challenge, node).map_err(|e| format!("baseline: {e:?}"))?;
    print_result("baseline", &baseline.result);
    println!("  baseline wall s: {:.1}", baseline.wall_s);
    if !probes {
        return Ok(());
    }
    let context = Context {
        map,
        challenge,
        node,
    };
    run_probes(&context, &baseline.samples, &baseline.result);
    Ok(())
}

fn write_fixed(args: &Args, challenges: &[Challenge]) -> Result<(), Box<dyn Error>> {
    let Some(path) = &args.fix else {
        return Ok(());
    };
    std::fs::write(path, serde_json::to_string_pretty(challenges)? + "\n")?;
    println!("wrote {path}");
    Ok(())
}
