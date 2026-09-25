use std::process::ExitCode;

use trapiks_mapgen::build::{BuildStats, build};
use trapiks_mapgen::input::{LoadStats, load};
use trapiks_mapgen::{stats::print, write::write};
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::MapData;

const USAGE: &str = "usage: trapiks-mapgen --out <output.bin.gz> (<input.json>... | --synthetic <cols>x<rows>@<spacing>)";

enum Source {
    Osm(Vec<String>),
    Synthetic(GridCity),
}

struct Args {
    out: String,
    source: Source,
}

fn parse_grid(value: &str) -> Option<GridCity> {
    let (size, spacing) = value.split_once('@')?;
    let (cols, rows) = size.split_once('x')?;
    Some(GridCity {
        cols: cols.parse().ok()?,
        rows: rows.parse().ok()?,
        spacing: spacing.parse().ok()?,
    })
}

fn parse_source(inputs: &[String]) -> Option<Source> {
    match inputs {
        [flag, grid] if flag == "--synthetic" => parse_grid(grid).map(Source::Synthetic),
        [] => None,
        _ => Some(Source::Osm(inputs.to_vec())),
    }
}

fn parse_args(args: &[String]) -> Option<Args> {
    let [flag, out, inputs @ ..] = args else {
        return None;
    };
    if flag != "--out" {
        return None;
    }
    Some(Args {
        out: out.clone(),
        source: parse_source(inputs)?,
    })
}

fn produce(source: &Source) -> Result<(LoadStats, BuildStats, MapData), String> {
    match source {
        Source::Synthetic(spec) => {
            Ok((LoadStats::default(), BuildStats::default(), grid_city(spec)))
        }
        Source::Osm(inputs) => {
            let (osm, load_stats) = load(inputs)?;
            let (map, build_stats) = build(&osm)?;
            Ok((load_stats, build_stats, map))
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    let (load_stats, build_stats, map) = produce(&args.source)?;
    let sizes = write(&map, &args.out)?;
    print(&load_stats, &build_stats, &map, &sizes);
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(args) = parse_args(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
