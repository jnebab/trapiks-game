use std::process::ExitCode;

use trapiks_mapgen::{build::build, input::load, stats::print, write::write};

const USAGE: &str = "usage: trapiks-mapgen --out <output.bin.gz> <input.json>...";

struct Args {
    out: String,
    inputs: Vec<String>,
}

fn parse_args(args: &[String]) -> Option<Args> {
    let [flag, out, inputs @ ..] = args else {
        return None;
    };
    if flag != "--out" || inputs.is_empty() {
        return None;
    }
    Some(Args {
        out: out.clone(),
        inputs: inputs.to_vec(),
    })
}

fn run(args: &Args) -> Result<(), String> {
    let (osm, load_stats) = load(&args.inputs)?;
    let map = build(&osm)?;
    let sizes = write(&map, &args.out)?;
    print(&load_stats, &map, &sizes);
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
