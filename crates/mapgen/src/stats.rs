use trapiks_sim_core::map::{Control, MapData};

use crate::input::LoadStats;
use crate::write::WriteSizes;

const CONTROL_KINDS: [Control; 4] = [
    Control::Signal,
    Control::AllWayStop,
    Control::Stop,
    Control::Yield,
];

pub fn print(load: &LoadStats, map: &MapData, sizes: &WriteSizes) {
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
    println!("raw bytes: {}", sizes.raw_bytes);
    println!("gzipped bytes: {}", sizes.gzipped_bytes);
}
