use std::io::Read;
use std::mem::size_of;
use std::time::Instant;

use flate2::read::GzDecoder;
use trapiks_sim_core::map::from_bytes;
use trapiks_sim_core::network::{Conflict, Movement, Network};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: build_junctions <map.bin.gz>")?;
    let mut bytes = Vec::new();
    GzDecoder::new(std::fs::File::open(path)?).read_to_end(&mut bytes)?;
    let map = from_bytes(&bytes)?;
    let mut network = Network::from_map(&map);
    let started = Instant::now();
    network.build_all_junctions();
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    report(&network, millis);
    Ok(())
}

fn report(network: &Network, millis: f64) {
    let node_count = network.nodes.count() as u32;
    let junctions: Vec<_> = (0..node_count)
        .filter_map(|n| network.junction(n))
        .collect();
    let movements: usize = junctions.iter().map(|j| j.movements.len()).sum();
    let conflicts: usize = junctions
        .iter()
        .flat_map(|j| &j.conflicts)
        .map(Vec::len)
        .sum();
    let live = (0..network.roads.count() as u32)
        .filter(|&r| network.roads.is_live(r))
        .count();
    let bytes = movements * size_of::<Movement>() + conflicts * size_of::<Conflict>();
    println!("nodes: {node_count}");
    println!("live roads: {live}");
    println!("junctions built: {}", junctions.len());
    println!("movements: {movements}");
    println!("conflicts: {conflicts}");
    println!("signal clusters: {}", network.signals().clusters().len());
    println!("build ms: {millis:.2}");
    println!("estimated bytes: {bytes}");
}
