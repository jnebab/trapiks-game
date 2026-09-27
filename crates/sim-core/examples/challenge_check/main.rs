mod args;
mod baseline;
mod expected;
mod probe;
mod site;

use std::error::Error;
use std::io::Read;

use flate2::read::GzDecoder;
use trapiks_sim_core::challenge::{Center, Challenge};
use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::{MapData, from_bytes};
use trapiks_sim_core::network::Network;

use args::Args;
use baseline::{print_result, run_baseline};
use expected::{expected_names, names_match};
use probe::Context;
use site::{Locator, MAX_CENTER_OFFSET_M, Site};

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse(std::env::args().skip(1))?;
    let map = load_map(&args.map)?;
    let mut challenges: Vec<Challenge> =
        serde_json::from_str(&std::fs::read_to_string(&args.challenges)?)?;
    let network = Network::from_map(&map);
    let locator = Locator {
        map: &map,
        network: &network,
        projection: Projection::from_origin(map.origin.clone()),
    };
    for challenge in challenges.iter_mut().filter(|c| selected(&args, c)) {
        override_demand(&args, challenge);
        check_challenge(&locator, challenge, args.fix.is_some())?;
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
}

fn check_challenge(
    locator: &Locator,
    challenge: &mut Challenge,
    fix: bool,
) -> Result<(), Box<dyn Error>> {
    println!(
        "challenge {} ({}) vph={} budget={}",
        challenge.id, challenge.name, challenge.vehicles_per_hour, challenge.budget
    );
    let (x, y) = challenge.center_xy(locator.map);
    let center = Vec2 { x, y };
    let expected = expected_names(&challenge.id);
    let nearest = locator
        .nearest(center)
        .ok_or("no junction with active degree >= 3")?;
    print_site("nearest junction", &nearest);
    let on_target =
        nearest.distance <= MAX_CENTER_OFFSET_M && names_match(expected, &nearest.names);
    println!(
        "  centre check: {}",
        if on_target { "PASS" } else { "FAIL" }
    );
    let best = locator.best_match(center, expected);
    match &best {
        Some(site) => print_site("best named candidate", site),
        None => println!("  best named candidate: none within 600 m matching {expected:?}"),
    }
    let node = best.as_ref().map_or(nearest.node, |site| site.node);
    if fix
        && !on_target
        && let Some(site) = &best
    {
        challenge.center = Center::LatLon {
            lat: site.lat,
            lon: site.lon,
        };
        println!("  fixed centre -> {:.6}, {:.6}", site.lat, site.lon);
    }
    run_checks(locator.map, challenge, node)
}

fn print_site(label: &str, site: &Site) {
    println!(
        "  {label}: node {} at {:.1} m, lat/lon {:.6}, {:.6}, roads {:?}",
        site.node, site.distance, site.lat, site.lon, site.names
    );
}

fn run_checks(map: &MapData, challenge: &Challenge, node: u32) -> Result<(), Box<dyn Error>> {
    let baseline = run_baseline(map, challenge, node).map_err(|e| format!("baseline: {e:?}"))?;
    print_result("baseline", &baseline.result);
    println!("  baseline wall s: {:.1}", baseline.wall_s);
    let context = Context {
        map,
        challenge,
        node,
    };
    for probe in context.probes(&baseline.movement_samples) {
        context.evaluate(&probe, &baseline.result);
    }
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
