use trapiks_sim_core::map::{AreaKind, Control, MapData};

use crate::build::BuildStats;
use crate::input::LoadStats;
use crate::write::WriteSizes;

const CONTROL_KINDS: [Control; 4] = [
    Control::Signal,
    Control::AllWayStop,
    Control::Stop,
    Control::Yield,
];

pub fn print(load: &LoadStats, build: &BuildStats, map: &MapData, sizes: &WriteSizes) {
    println!("dropped ways (geometry mismatch): {}", load.dropped_ways);
    println!("nodes: {}", map.node_count());
    println!("roads: {}", map.road_count());
    println!("points: {}", map.points.x.len());
    println!("names: {}", map.names.len());
    println!("turn bans: {}", map.turn_bans.len());
    for kind in CONTROL_KINDS {
        let count = map.nodes.control.iter().filter(|&&c| c == kind).count();
        println!("controlled nodes ({kind:?}): {count}");
    }
    for kind in [AreaKind::Water, AreaKind::Park] {
        let count = map.areas.kind.iter().filter(|&&k| k == kind).count();
        println!("area rings ({kind:?}): {count}");
    }
    println!("dropped multipolygon chains: {}", build.dropped_chains);
    println!(
        "dropped coastline pieces: {}",
        build.dropped_coastline_pieces
    );
    println!("raw bytes: {}", sizes.raw_bytes);
    println!("gzipped bytes: {}", sizes.gzipped_bytes);
}
