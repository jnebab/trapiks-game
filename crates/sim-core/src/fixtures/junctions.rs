use crate::map::{Control, MapData, RoadClass};

use super::{MapBuilder, RoadSpec};

pub fn four_way(lanes: u8, arm: f32) -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    b.control(centre, Control::Signal);
    let arms = [(0.0, -arm), (arm, 0.0), (0.0, arm), (-arm, 0.0)].map(|(x, y)| b.node(x, y));
    for node in arms {
        b.road(
            node,
            centre,
            RoadSpec::new(RoadClass::Primary, lanes, lanes),
        );
    }
    b.build()
}

pub fn t_junction() -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    let west = b.node(-200.0, 0.0);
    let east = b.node(200.0, 0.0);
    let south = b.node(0.0, 150.0);
    let major = RoadSpec::new(RoadClass::Primary, 2, 2);
    b.road(west, centre, major.clone());
    b.road(centre, east, major);
    b.road(south, centre, RoadSpec::new(RoadClass::Residential, 1, 1));
    b.build()
}

pub fn dual_carriageway_cross() -> MapData {
    let mut b = MapBuilder::new();
    let positions = [
        (-10.0, -10.0),
        (10.0, -10.0),
        (10.0, 10.0),
        (-10.0, 10.0),
        (-210.0, 10.0),
        (210.0, 10.0),
        (210.0, -10.0),
        (-210.0, -10.0),
        (-10.0, -210.0),
        (-10.0, 210.0),
        (10.0, 210.0),
        (10.0, -210.0),
    ];
    let n = positions.map(|(x, y)| b.node(x, y));
    for &node in &n[..4] {
        b.control(node, Control::Signal);
    }
    let pairs = [
        (4, 3),
        (3, 2),
        (2, 5),
        (6, 1),
        (1, 0),
        (0, 7),
        (8, 0),
        (0, 3),
        (3, 9),
        (10, 2),
        (2, 1),
        (1, 11),
    ];
    for (from, to) in pairs {
        b.road(n[from], n[to], RoadSpec::new(RoadClass::Trunk, 3, 0));
    }
    b.build()
}

pub fn one_way_pair() -> MapData {
    let mut b = MapBuilder::new();
    let positions = [
        (0.0, 0.0),
        (100.0, 0.0),
        (-200.0, 0.0),
        (300.0, 0.0),
        (0.0, -200.0),
        (0.0, 200.0),
        (100.0, 200.0),
        (100.0, -200.0),
    ];
    let n = positions.map(|(x, y)| b.node(x, y));
    let street = RoadSpec::new(RoadClass::Tertiary, 1, 1);
    let one_way = RoadSpec::new(RoadClass::Secondary, 2, 0);
    b.road(n[2], n[0], street.clone());
    b.road(n[0], n[1], street.clone());
    b.road(n[1], n[3], street);
    b.road(n[4], n[0], one_way.clone());
    b.road(n[0], n[5], one_way.clone());
    b.road(n[6], n[1], one_way.clone());
    b.road(n[1], n[7], one_way);
    b.build()
}

pub fn dead_end() -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    let west = b.node(-150.0, 0.0);
    let east = b.node(150.0, 0.0);
    let end = b.node(0.0, 100.0);
    let street = RoadSpec::new(RoadClass::Tertiary, 1, 1);
    b.road(west, centre, street.clone());
    b.road(centre, east, street);
    b.road(centre, end, RoadSpec::new(RoadClass::Residential, 1, 1));
    b.build()
}
