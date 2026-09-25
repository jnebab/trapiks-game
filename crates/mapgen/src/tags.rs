use trapiks_sim_core::map::RoadClass;

use crate::osm::Tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oneway {
    No,
    Forward,
    Backward,
}

const MAX_LANES: u8 = 8;

pub fn oneway(tags: &Tags, class: RoadClass) -> Oneway {
    match tag(tags, "oneway") {
        Some("yes" | "true" | "1") => return Oneway::Forward,
        Some("-1" | "reverse") => return Oneway::Backward,
        Some("no") => return Oneway::No,
        _ => {}
    }
    let roundabout = matches!(tag(tags, "junction"), Some("roundabout" | "circular"));
    if roundabout || class.implies_oneway() {
        return Oneway::Forward;
    }
    Oneway::No
}

pub fn lanes(tags: &Tags, class: RoadClass, oneway: Oneway) -> (u8, u8) {
    let total = parse_lanes(tags, "lanes");
    let travel = || clamp_lanes(total.unwrap_or(class.default_lanes()));
    match oneway {
        Oneway::Forward => (travel(), 0),
        Oneway::Backward => (0, travel()),
        Oneway::No => {
            let (forward, backward) = two_way_lanes(tags, class, total);
            (clamp_lanes(forward), clamp_lanes(backward))
        }
    }
}

fn two_way_lanes(tags: &Tags, class: RoadClass, total: Option<u8>) -> (u8, u8) {
    let forward = parse_lanes(tags, "lanes:forward");
    let backward = parse_lanes(tags, "lanes:backward");
    let default = class.default_lanes();
    match (forward, backward, total) {
        (Some(f), Some(b), _) => (f, b),
        (Some(f), None, Some(n)) => (f, remainder(n, f)),
        (None, Some(b), Some(n)) => (remainder(n, b), b),
        (None, None, Some(n)) => (n.div_ceil(2).max(1), (n / 2).max(1)),
        (f, b, None) => (f.unwrap_or(default), b.unwrap_or(default)),
    }
}

fn remainder(total: u8, tagged: u8) -> u8 {
    total.saturating_sub(tagged).max(1)
}

fn clamp_lanes(count: u8) -> u8 {
    count.clamp(1, MAX_LANES)
}

fn parse_lanes(tags: &Tags, key: &str) -> Option<u8> {
    tag(tags, key)?.trim().parse().ok()
}

pub fn speed_kph(tags: &Tags, class: RoadClass) -> u8 {
    let parsed = tag(tags, "maxspeed").and_then(parse_speed);
    let kph = parsed.unwrap_or(u32::from(class.default_speed_kph()));
    let clamped = kph.clamp(5, 130);
    u8::try_from(clamped).unwrap_or(130)
}

fn parse_speed(value: &str) -> Option<u32> {
    let value = value.trim();
    if let Some(mph) = value.strip_suffix("mph") {
        let mph: u16 = mph.trim().parse().ok()?;
        return Some((u32::from(mph) * 1609 + 500) / 1000);
    }
    let kph = value.strip_suffix("km/h").unwrap_or(value);
    kph.trim().parse().ok()
}

pub fn layer(tags: &Tags) -> i8 {
    if let Some(layer) = tag(tags, "layer").and_then(|v| v.trim().parse::<i32>().ok()) {
        return i8::try_from(layer.clamp(-3, 5)).unwrap_or(0);
    }
    if is_set(tags, "bridge") {
        return 1;
    }
    if is_set(tags, "tunnel") {
        return -1;
    }
    0
}

fn is_set(tags: &Tags, key: &str) -> bool {
    tag(tags, key).is_some_and(|value| value != "no")
}

pub fn name(tags: &Tags) -> Option<&str> {
    tag(tags, "name").or_else(|| tag(tags, "ref"))
}

pub fn tag<'a>(tags: &'a Tags, key: &str) -> Option<&'a str> {
    tags.get(key).map(String::as_str)
}
