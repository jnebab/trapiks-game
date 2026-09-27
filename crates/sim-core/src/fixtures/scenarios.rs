use crate::config::{SimConfig, SimMode};
use crate::network::Network;
use crate::rng::Pcg32;
use crate::sim::Sim;

use super::{GridCity, four_way, grid_city};

pub const FOUR_WAY_HASH_5000: u64 = 0xdceb_7b79_899f_aae7;
pub const GRID_CITY_HASH_1000: u64 = 0x28f6_3482_57ba_4981;

pub fn four_way_5000() -> u64 {
    let map = four_way(2, 200.0);
    let mut network = Network::from_map(&map);
    let movements: Vec<[u32; 2]> = network
        .ensure_junction(0)
        .movements
        .iter()
        .map(|m| [m.from_link, m.to_link])
        .collect();
    let mut sim = Sim::new(&map, 7);
    let mut schedule_rng = Pcg32::new(99, 1);
    for tick in 0..5_000 {
        if tick % 10 == 0 {
            let pick = schedule_rng.below(12) as usize;
            if let Some(route) = movements.get(pick) {
                let _ = sim.spawn(route);
            }
        }
        sim.step();
    }
    sim.state_hash()
}

pub fn grid_city_demand_1000() -> u64 {
    let map = grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed: 5,
        mode: SimMode::City,
        vehicles_per_hour: 8_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    for _ in 0..1_000 {
        sim.step();
    }
    sim.state_hash()
}
