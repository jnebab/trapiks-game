use std::process::ExitCode;

const USAGE: &str = "usage: trapiks-mapgen --out <output.bin.gz> <input.json>...";

fn is_valid(args: &[String]) -> bool {
    args.len() == 3 && args[0] == "--out"
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if is_valid(&args) {
        return ExitCode::SUCCESS;
    }
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
