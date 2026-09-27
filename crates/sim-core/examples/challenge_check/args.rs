pub struct Args {
    pub map: String,
    pub challenges: String,
    pub only: Option<String>,
    pub vph: Option<f64>,
    pub fix: Option<String>,
    pub sites_only: bool,
    pub probes: bool,
}

const USAGE: &str = "usage: challenge_check <map.bin.gz> [challenges.json] [--only <id>] [--vph <n>] [--fix <out.json>] [--sites-only] [--no-probes]";
const DEFAULT_CHALLENGES: &str = "web/public/challenges/metro-manila.json";

pub fn parse(raw: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut positional = Vec::new();
    let mut args = Args {
        map: String::new(),
        challenges: DEFAULT_CHALLENGES.to_string(),
        only: None,
        vph: None,
        fix: None,
        sites_only: false,
        probes: true,
    };
    let mut raw = raw.peekable();
    while let Some(arg) = raw.next() {
        match arg.as_str() {
            "--only" => args.only = Some(value(&mut raw)?),
            "--vph" => args.vph = Some(value(&mut raw)?.parse().map_err(|_| USAGE.to_string())?),
            "--sites-only" => args.sites_only = true,
            "--no-probes" => args.probes = false,
            "--fix" => args.fix = Some(value(&mut raw)?),
            _ => positional.push(arg),
        }
    }
    apply_positional(&mut args, positional)?;
    Ok(args)
}

fn value(raw: &mut impl Iterator<Item = String>) -> Result<String, String> {
    raw.next().ok_or_else(|| USAGE.to_string())
}

fn apply_positional(args: &mut Args, positional: Vec<String>) -> Result<(), String> {
    let mut positional = positional.into_iter();
    args.map = positional.next().ok_or_else(|| USAGE.to_string())?;
    if let Some(challenges) = positional.next() {
        args.challenges = challenges;
    }
    match positional.next() {
        Some(_) => Err(USAGE.to_string()),
        None => Ok(()),
    }
}
