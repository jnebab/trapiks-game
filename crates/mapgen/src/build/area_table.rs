use trapiks_sim_core::map::{AreaKind, AreaTable};

use super::BuildStats;
use crate::areas::{AreaRings, closed_way_ring, relation_rings};
use crate::coastline::{LonLatBox, sea_rings};
use crate::input::{OsmData, OsmWay};
use crate::osm::LatLon;
use crate::polygons::{Polygon, finish_polygons};
use crate::tags::tag;
use trapiks_sim_core::geo::Projection;

#[derive(Default)]
struct Groups {
    water: Vec<AreaRings>,
    park: Vec<AreaRings>,
    stats: BuildStats,
}

impl Groups {
    fn push(&mut self, kind: AreaKind, rings: AreaRings) {
        match kind {
            AreaKind::Water => self.water.push(rings),
            AreaKind::Park => self.park.push(rings),
        }
    }
}

pub fn area_table(
    osm: &OsmData,
    bbox: &LonLatBox,
    projection: &Projection,
) -> (AreaTable, BuildStats) {
    let mut groups = Groups::default();
    for way in osm.ways.values() {
        if let Some((kind, ring)) = closed_way_ring(way, &osm.nodes) {
            groups.push(kind, single(ring));
        }
    }
    for relation in osm.relations.values() {
        let Some(found) = relation_rings(relation) else {
            continue;
        };
        groups.stats.dropped_chains += found.dropped;
        groups.push(found.kind, found.rings);
    }
    let coastline: Vec<&OsmWay> = osm
        .ways
        .values()
        .filter(|way| tag(&way.tags, "natural") == Some("coastline"))
        .collect();
    let (sea, dropped) = sea_rings(&coastline, &osm.nodes, bbox);
    groups.stats.dropped_coastline_pieces = dropped;
    groups.water.extend(sea.into_iter().map(single));
    let mut table = AreaTable {
        ring_start: vec![0],
        ..AreaTable::default()
    };
    for (kind, group) in [
        (AreaKind::Water, &groups.water),
        (AreaKind::Park, &groups.park),
    ] {
        for rings in group {
            push_polygons(&mut table, kind, finish_polygons(rings, projection));
        }
    }
    (table, groups.stats)
}

fn single(ring: Vec<LatLon>) -> AreaRings {
    AreaRings {
        outers: vec![ring],
        inners: Vec::new(),
    }
}

fn push_polygons(table: &mut AreaTable, kind: AreaKind, polygons: Vec<Polygon>) {
    for polygon in polygons {
        push_ring(table, kind, false, &polygon.outer);
        for hole in &polygon.holes {
            push_ring(table, kind, true, hole);
        }
    }
}

fn push_ring(table: &mut AreaTable, kind: AreaKind, hole: bool, ring: &[(f32, f32)]) {
    table.kind.push(kind);
    table.hole.push(hole);
    for &(x, y) in ring {
        table.x.push(x);
        table.y.push(y);
    }
    table
        .ring_start
        .push(u32::try_from(table.x.len()).unwrap_or(u32::MAX));
}
