use crate::map::{MapData, RoadClass};

use super::{MapBuilder, RoadSpec};

pub fn corridor() -> MapData {
    let mut b = MapBuilder::new();
    let nodes = [0.0, 400.0, 800.0, 1200.0].map(|x| b.node(x, 0.0));
    for pair in nodes.windows(2) {
        b.road(pair[0], pair[1], RoadSpec::new(RoadClass::Primary, 2, 2));
    }
    b.build()
}
