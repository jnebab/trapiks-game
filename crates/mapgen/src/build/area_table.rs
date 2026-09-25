use trapiks_sim_core::map::{AreaKind, AreaTable};

use super::BuildStats;
use crate::areas::{closed_way_ring, finish, relation_rings};
use crate::coastline::{LonLatBox, sea_rings};
use crate::input::{OsmData, OsmWay};
use crate::osm::LatLon;
use crate::project::Projection;
use crate::tags::tag;

#[derive(Default)]
struct Groups {
    water: Vec<Vec<LatLon>>,
    park: Vec<Vec<LatLon>>,
    stats: BuildStats,
}

impl Groups {
    fn push(&mut self, kind: AreaKind, ring: Vec<LatLon>) {
        match kind {
            AreaKind::Water => self.water.push(ring),
            AreaKind::Park => self.park.push(ring),
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
            groups.push(kind, ring);
        }
    }
    for relation in osm.relations.values() {
        let Some((kind, rings, dropped)) = relation_rings(relation) else {
            continue;
        };
        groups.stats.dropped_chains += dropped;
        rings.into_iter().for_each(|ring| groups.push(kind, ring));
    }
    let coastline: Vec<&OsmWay> = osm
        .ways
        .values()
        .filter(|way| tag(&way.tags, "natural") == Some("coastline"))
        .collect();
    let (sea, dropped) = sea_rings(&coastline, &osm.nodes, bbox);
    groups.stats.dropped_coastline_pieces = dropped;
    groups.water.extend(sea);
    let mut table = AreaTable {
        ring_start: vec![0],
        ..AreaTable::default()
    };
    push_rings(
        &mut table,
        AreaKind::Water,
        finish(groups.water, projection),
    );
    push_rings(&mut table, AreaKind::Park, finish(groups.park, projection));
    (table, groups.stats)
}

fn push_rings(table: &mut AreaTable, kind: AreaKind, rings: Vec<Vec<(f32, f32)>>) {
    for ring in rings {
        table.kind.push(kind);
        for (x, y) in ring {
            table.x.push(x);
            table.y.push(y);
        }
        table
            .ring_start
            .push(u32::try_from(table.x.len()).unwrap_or(u32::MAX));
    }
}
