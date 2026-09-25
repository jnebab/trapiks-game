use trapiks_mapgen::build::build;
use trapiks_mapgen::input::{OsmData, OsmNode, OsmRelation, OsmWay};
use trapiks_mapgen::osm::{Member, Tags};
use trapiks_sim_core::map::MapData;

pub const LAT: f64 = 14.6;
pub const LON: f64 = 121.0;
pub const STEP: f64 = 0.0001;
const ARM: f64 = 6.0;

#[derive(Default)]
pub struct Fixture {
    pub osm: OsmData,
}

fn tags(pairs: &[(&str, &str)]) -> Tags {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn member(kind: &str, id: i64, role: &str) -> Member {
    Member {
        kind: kind.to_string(),
        id,
        role: role.to_string(),
    }
}

impl Fixture {
    pub fn node(&mut self, id: i64, lat: f64, lon: f64) -> &mut Self {
        self.tagged_node(id, lat, lon, &[])
    }

    pub fn tagged_node(
        &mut self,
        id: i64,
        lat: f64,
        lon: f64,
        pairs: &[(&str, &str)],
    ) -> &mut Self {
        let node = OsmNode {
            lat,
            lon,
            tags: tags(pairs),
        };
        self.osm.nodes.insert(id, node);
        self
    }

    pub fn way(&mut self, id: i64, nodes: &[i64], pairs: &[(&str, &str)]) -> &mut Self {
        let way = OsmWay {
            nodes: nodes.to_vec(),
            tags: tags(pairs),
        };
        self.osm.ways.insert(id, way);
        self
    }

    pub fn relation(&mut self, (id, relation): (i64, OsmRelation)) -> &mut Self {
        self.osm.relations.insert(id, relation);
        self
    }

    pub fn build(&self) -> MapData {
        match build(&self.osm) {
            Ok(map) => map,
            Err(error) => panic!("build failed: {error}"),
        }
    }

    pub fn row(&mut self, first_id: i64, count: i64) -> &mut Self {
        for i in 0..count {
            self.node(first_id + i, LAT, LON + STEP * i as f64);
        }
        self
    }
}

pub fn single_way_map(pairs: &[(&str, &str)]) -> MapData {
    let mut fixture = Fixture::default();
    fixture.row(1, 3).way(10, &[1, 2, 3], pairs);
    fixture.build()
}

pub fn four_way(fixture: &mut Fixture) -> &mut Fixture {
    fixture
        .node(1, LAT, LON - ARM * STEP)
        .node(2, LAT, LON + ARM * STEP)
        .node(3, LAT - ARM * STEP, LON)
        .node(4, LAT + ARM * STEP, LON)
        .node(5, LAT, LON)
        .way(101, &[1, 5], &[("highway", "residential")])
        .way(102, &[5, 2], &[("highway", "residential")])
        .way(201, &[3, 5], &[("highway", "residential")])
        .way(202, &[5, 4], &[("highway", "residential")])
}

pub fn restriction(
    id: i64,
    kind: &str,
    from_way: i64,
    via_node: i64,
    to_way: i64,
) -> (i64, OsmRelation) {
    let relation = OsmRelation {
        members: vec![
            member("way", from_way, "from"),
            member("node", via_node, "via"),
            member("way", to_way, "to"),
        ],
        tags: tags(&[("type", "restriction"), ("restriction", kind)]),
    };
    (id, relation)
}
