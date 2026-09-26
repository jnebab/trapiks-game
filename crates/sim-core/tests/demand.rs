use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::consts::TRIP_EXPIRY_TICKS;
use trapiks_sim_core::demand::{LinkQueues, RoutedTrip, Trip};
use trapiks_sim_core::fixtures::{GridCity, MapBuilder, RoadSpec, corridor, four_way, grid_city};
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{LinkId, Network, road_of};
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, SpawnError};

const SAMPLES: u32 = 100_000;

fn star() -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    let east = b.node(300.0, 0.0);
    let north = b.node(0.0, 500.0);
    let west = b.node(-800.0, 0.0);
    b.road(centre, east, RoadSpec::new(RoadClass::Primary, 1, 1));
    b.road(centre, north, RoadSpec::new(RoadClass::Residential, 1, 1));
    b.road(centre, west, RoadSpec::new(RoadClass::Trunk, 1, 1));
    b.build()
}

fn base_weight(network: &Network, link: LinkId) -> f64 {
    let (start, end) = network.link_span(link);
    let class = network.roads.class[road_of(link) as usize];
    (end - start).max(0.1) * class.demand_weight()
}

#[test]
fn demand_weights() {
    let map = star();
    let sim = Sim::new(&map, 1);
    let network = sim.network();
    let origins = &sim.demand_tables().origins;
    assert_eq!(origins.links.len(), 6);
    let total: f64 = origins.links.iter().map(|&l| base_weight(network, l)).sum();
    let mut counts = [0u32; 6];
    let mut rng = Pcg32::new(5, 9);
    for _ in 0..SAMPLES {
        let link = origins.sample(&mut rng).expect("origin");
        counts[link as usize] += 1;
    }
    for (link, &count) in counts.iter().enumerate() {
        let expected = base_weight(network, link as LinkId) / total;
        let observed = f64::from(count) / f64::from(SAMPLES);
        assert!(
            (observed - expected).abs() <= 0.005,
            "{link}: {observed} {expected}"
        );
    }
}

fn corridor_sim(vph: f64) -> Sim {
    let config = SimConfig {
        seed: 3,
        mode: SimMode::City,
        vehicles_per_hour: vph,
        budget: None,
    };
    Sim::from_config(&corridor(), &config)
}

#[test]
fn poisson_rate() {
    let mut sim = corridor_sim(3_600.0);
    for _ in 0..36_000 {
        sim.step();
    }
    let created = sim.stats().created as f64;
    assert!((created - 3_600.0).abs() <= 180.0, "{created}");
    let mut idle = corridor_sim(0.0);
    for _ in 0..1_000 {
        idle.step();
    }
    assert_eq!(idle.stats().created, 0);
}

#[test]
fn trip_band() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    sim.set_demand(7_200.0);
    for _ in 0..3_000 {
        sim.step();
    }
    let stats = sim.stats();
    assert!(stats.created > 0);
    assert_eq!(stats.unserved, stats.created);
    assert_eq!(stats.spawned, 0);
}

fn routed(from: LinkId, created_tick: u64) -> RoutedTrip {
    RoutedTrip {
        trip: Trip {
            from,
            to: 99,
            created_tick,
        },
        route: vec![from, 99],
    }
}

#[test]
fn queue_expiry() {
    let mut queues = LinkQueues::default();
    let fed: Vec<(LinkId, u64)> = vec![(4, 0), (4, 10), (2, 5), (7, 300), (4, 10)];
    let mut dropped_at: Vec<u64> = Vec::new();
    let mut fed_count = 0;
    let mut unserved = 0;
    for tick in 0..2_000u64 {
        for &(from, created) in fed.iter().filter(|&&(_, created)| created == tick) {
            queues.push(routed(from, created));
            fed_count += 1;
        }
        let outcome = queues.step(tick, |_| Err(SpawnError::Blocked));
        assert_eq!(outcome.spawned, 0);
        dropped_at.extend(std::iter::repeat_n(tick, outcome.unserved as usize));
        unserved += outcome.unserved;
    }
    let mut expected: Vec<u64> = fed.iter().map(|&(_, c)| c + TRIP_EXPIRY_TICKS).collect();
    expected.sort_unstable();
    assert_eq!(dropped_at, expected);
    assert_eq!(unserved, fed_count);
    assert_eq!(queues.len(), 0);
}

fn region_config(vph: f64) -> SimConfig {
    SimConfig {
        seed: 11,
        mode: SimMode::Region {
            center_x: 1_500.0,
            center_y: 1_500.0,
            radius: 600.0,
        },
        vehicles_per_hour: vph,
        budget: None,
    }
}

fn region_map() -> MapData {
    grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    })
}

fn place_link(place: Place, sim: &Sim, slot: u32) -> Option<LinkId> {
    match place {
        Place::Link { link, .. } => Some(link),
        Place::Movement { .. } => sim.vehicles().route_link(slot, 1),
    }
}

#[test]
fn region_mode_only_active_links() {
    let mut sim = Sim::from_config(&region_map(), &region_config(3_600.0));
    let mut seen = 0;
    for _ in 0..3_000 {
        sim.step();
        let vehicles = sim.vehicles();
        for slot in vehicles.live_slots() {
            let link = place_link(vehicles.place[slot as usize], &sim, slot).expect("link");
            assert!(sim.network().is_link_active(link));
            seen += 1;
        }
    }
    assert!(seen > 0);
    assert!(sim.stats().arrivals > 0);
}

#[test]
fn region_mode_boosts_sources() {
    let sim = Sim::from_config(&region_map(), &region_config(0.0));
    let network = sim.network();
    let sources = &network.region().expect("region").sources;
    let origins = &sim.demand_tables().origins;
    let plain_total: f64 = origins.links.iter().map(|&l| base_weight(network, l)).sum();
    let plain_sources: f64 = origins
        .links
        .iter()
        .filter(|l| sources.binary_search(l).is_ok())
        .map(|&l| base_weight(network, l))
        .sum();
    let mut rng = Pcg32::new(21, 4);
    let hits = (0..SAMPLES)
        .filter_map(|_| origins.sample(&mut rng))
        .filter(|l| sources.binary_search(l).is_ok())
        .count();
    let observed = hits as f64 / f64::from(SAMPLES);
    let plain = plain_sources / plain_total;
    assert!(observed >= 3.0 * plain, "{observed} {plain}");
}

#[test]
fn region_mode_sink_arrival() {
    let mut sim = Sim::from_config(&region_map(), &region_config(0.0));
    let mask = sim.network().region().expect("region").clone();
    let (from, to) = mask
        .sources
        .iter()
        .flat_map(|&from| mask.sinks.iter().map(move |&to| (from, to)))
        .find(|&(from, to)| from != to && road_of(from) != road_of(to))
        .expect("pair");
    let routed = sim.spawn_trip(from, to);
    assert!(routed.is_ok(), "{routed:?}");
    for _ in 0..6_000 {
        sim.step();
        if sim.vehicle_count() == 0 {
            break;
        }
    }
    let stats = sim.stats();
    assert_eq!(sim.vehicle_count(), 0);
    assert_eq!(stats.arrivals, 1);
    assert_eq!(stats.stranded, 0);
}
