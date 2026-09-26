mod support;

use support::{Fixture, LAT, LON};
use trapiks_mapgen::areas::{closed_way_ring, finish, relation_rings};
use trapiks_mapgen::project::Projection;
use trapiks_sim_core::map::AreaKind;

const D: f64 = 0.001;

fn square(fixture: &mut Fixture, first_id: i64) -> [i64; 4] {
    let ids = [first_id, first_id + 1, first_id + 2, first_id + 3];
    let corners = [(0.0, 0.0), (0.0, D), (D, D), (D, 0.0)];
    for (id, (dlat, dlon)) in ids.iter().zip(corners) {
        fixture.node(*id, LAT + dlat, LON + dlon);
    }
    ids
}

fn projection() -> Projection {
    Projection::centered(LAT, LON, LAT + D, LON + D)
}

#[test]
fn closed_way_area() {
    let mut fixture = Fixture::default();
    let [a, b, c, d] = square(&mut fixture, 1);
    fixture
        .way(10, &[a, b, c, d, a], &[("natural", "water")])
        .way(11, &[a, b, c, d, a], &[("leisure", "park")]);
    let osm = &fixture.osm;
    let Some((kind, ring)) = closed_way_ring(&osm.ways[&10], &osm.nodes) else {
        panic!("no water ring");
    };
    assert_eq!(kind, AreaKind::Water);
    let rings = finish(vec![ring], &projection());
    assert_eq!(rings.len(), 1);
    assert_eq!(rings[0].len(), 4);
    let park = closed_way_ring(&osm.ways[&11], &osm.nodes).map(|(kind, _)| kind);
    assert_eq!(park, Some(AreaKind::Park));
}

#[test]
fn closed_way_area_in_build() {
    let mut fixture = Fixture::default();
    let [a, b, c, d] = square(&mut fixture, 1);
    fixture
        .way(10, &[a, b, c, d, a], &[("leisure", "park")])
        .way(11, &[a, b, c, d, a], &[("natural", "water")])
        .row(100, 2)
        .way(20, &[100, 101], &[("highway", "residential")]);
    let map = fixture.build();
    assert_eq!(map.areas.kind, vec![AreaKind::Water, AreaKind::Park]);
    assert_eq!(map.area_ring(0).0.len(), 4);
}

#[test]
fn multipolygon_outer_rings() {
    let mut fixture = Fixture::default();
    let [a, b, c, d] = square(&mut fixture, 1);
    let [e, f, g, _] = square(&mut fixture, 11);
    fixture
        .way(10, &[a, b, c], &[])
        .way(11, &[a, d, c], &[])
        .way(12, &[e, f, g], &[])
        .multipolygon(
            50,
            &[("outer", 10), ("outer", 11), ("inner", 12)],
            &[("type", "multipolygon"), ("natural", "water")],
        )
        .multipolygon(
            51,
            &[("outer", 12)],
            &[("type", "multipolygon"), ("leisure", "park")],
        );
    let relations = &fixture.osm.relations;
    let Some((kind, rings, dropped)) = relation_rings(&relations[&50]) else {
        panic!("no multipolygon");
    };
    assert_eq!((kind, rings.len(), dropped), (AreaKind::Water, 1, 0));
    assert_eq!(rings[0].len(), 5);
    assert_eq!(finish(rings, &projection())[0].len(), 4);
    let Some((_, rings, dropped)) = relation_rings(&relations[&51]) else {
        panic!("no park multipolygon");
    };
    assert_eq!((rings.len(), dropped), (0, 1));
}
