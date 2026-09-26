mod support;

use support::Fixture;
use trapiks_mapgen::areas::finish;
use trapiks_mapgen::coastline::{LonLatBox, sea_rings};
use trapiks_mapgen::input::OsmWay;
use trapiks_mapgen::osm::LatLon;
use trapiks_mapgen::simplify::ring_area;
use trapiks_sim_core::geo::Projection;

const BOX: LonLatBox = LonLatBox {
    min_lon: 120.95,
    min_lat: 14.55,
    max_lon: 121.05,
    max_lat: 14.65,
};

fn projection() -> Projection {
    Projection::centered(BOX.min_lat, BOX.min_lon, BOX.max_lat, BOX.max_lon)
}

fn coastline(ways: &[&[(f64, f64)]]) -> (Vec<Vec<LatLon>>, u32) {
    let mut fixture = Fixture::default();
    let mut next_node = 1;
    for (way_id, points) in (1..).zip(ways) {
        let mut ids = Vec::new();
        for (lon, lat) in points.iter() {
            fixture.node(next_node, *lat, *lon);
            ids.push(next_node);
            next_node += 1;
        }
        fixture.way(way_id, &ids, &[("natural", "coastline")]);
    }
    let ways: Vec<&OsmWay> = fixture.osm.ways.values().collect();
    sea_rings(&ways, &fixture.osm.nodes, &BOX)
}

fn projected_area(rings: Vec<Vec<LatLon>>) -> f64 {
    finish(rings, &projection())
        .iter()
        .map(|ring| {
            let points: Vec<(f64, f64)> = ring
                .iter()
                .map(|&(x, y)| (f64::from(x), f64::from(y)))
                .collect();
            ring_area(&points)
        })
        .sum()
}

fn half_box_area() -> f64 {
    let p = projection();
    let (x0, y0) = p.project(BOX.min_lat, BOX.min_lon);
    let (x1, y1) = p.project(BOX.max_lat, BOX.max_lon);
    ((x1 - x0) * (y1 - y0)).abs() / 2.0
}

fn has(ring: &[LatLon], lon: f64, lat: f64) -> bool {
    ring.contains(&LatLon { lat, lon })
}

fn corner_count(ring: &[LatLon]) -> usize {
    let lons = [BOX.min_lon, BOX.max_lon];
    let lats = [BOX.min_lat, BOX.max_lat];
    lons.iter()
        .flat_map(|lon| lats.iter().map(move |lat| (*lon, *lat)))
        .filter(|(lon, lat)| has(ring, *lon, *lat))
        .count()
}

#[test]
fn coastline_single_piece() {
    let (rings, dropped) = coastline(&[
        &[(121.0, 14.66), (121.0, 14.60)],
        &[(121.0, 14.60), (121.0, 14.54)],
    ]);
    assert_eq!((rings.len(), dropped), (1, 0));
    assert!(has(&rings[0], BOX.min_lon, BOX.max_lat));
    assert!(has(&rings[0], BOX.min_lon, BOX.min_lat));
    let area = projected_area(rings);
    assert!((area / half_box_area() - 1.0).abs() < 0.01);
}

#[test]
fn coastline_corner_wrap() {
    let (rings, dropped) = coastline(&[&[(120.99, 14.66), (120.94, 14.61)]]);
    assert_eq!((rings.len(), dropped), (1, 0));
    assert!(has(&rings[0], BOX.min_lon, BOX.max_lat));
    assert_eq!(corner_count(&rings[0]), 1);
}

#[test]
fn coastline_two_pieces() {
    let (rings, dropped) = coastline(&[
        &[(120.975, 14.66), (120.975, 14.54)],
        &[(121.025, 14.54), (121.025, 14.66)],
    ]);
    assert_eq!((rings.len(), dropped), (2, 0));
    assert_eq!((rings[0].len(), rings[1].len()), (4, 4));
    assert!(has(&rings[0], BOX.min_lon, BOX.max_lat));
    assert!(has(&rings[0], BOX.min_lon, BOX.min_lat));
    assert!(has(&rings[1], BOX.max_lon, BOX.max_lat));
    assert!(has(&rings[1], BOX.max_lon, BOX.min_lat));
    let area = projected_area(rings);
    assert!((area / half_box_area() - 1.0).abs() < 0.01);
}

#[test]
fn coastline_revisit_is_dropped() {
    let (rings, dropped) = coastline(&[
        &[(120.975, 14.66), (120.975, 14.54)],
        &[(121.025, 14.66), (121.025, 14.54)],
    ]);
    assert!(dropped > 0);
    assert!(rings.len() <= 1);
}

#[test]
fn coastline_island_is_ignored() {
    let (rings, dropped) = coastline(&[&[
        (121.0, 14.6),
        (121.01, 14.6),
        (121.01, 14.61),
        (121.0, 14.6),
    ]]);
    assert_eq!((rings.len(), dropped), (0, 0));
}

#[test]
fn coastline_ending_inside_is_dropped() {
    let (rings, dropped) = coastline(&[&[(121.0, 14.66), (121.0, 14.60)]]);
    assert_eq!((rings.len(), dropped), (0, 1));
}

#[test]
fn coastline_segment_ending_on_edge_splits_pieces() {
    let (rings, dropped) = coastline(&[&[
        (121.0, 14.66),
        (121.05, 14.6),
        (121.06, 14.6),
        (121.06, 14.56),
        (121.0, 14.55),
        (121.0, 14.54),
    ]]);
    assert_eq!((rings.len(), dropped), (1, 0));
    assert_eq!(rings[0].len(), 6);
    assert!(has(&rings[0], 121.05, 14.6));
    assert!(has(&rings[0], 121.0, 14.55));
}

#[test]
fn coastline_touching_edge_and_returning_keeps_one_piece() {
    let (rings, dropped) = coastline(&[&[
        (120.94, 14.6),
        (121.05, 14.6),
        (121.0, 14.62),
        (121.0, 14.66),
    ]]);
    assert_eq!((rings.len(), dropped), (1, 0));
    assert_eq!(rings[0].len(), 7);
    assert!(has(&rings[0], 121.05, 14.6));
    assert!(has(&rings[0], 120.95, 14.6));
}
