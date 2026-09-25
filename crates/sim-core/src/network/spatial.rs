use crate::consts::SPATIAL_CELL;
use crate::geom::{Bounds, Vec2, bounds};

use super::node::NodeStore;
use super::road::RoadStore;

#[derive(Clone, Debug, Default)]
struct CellTable {
    cell_start: Vec<u32>,
    ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Frame {
    origin: Vec2,
    cols: usize,
    rows: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SpatialGrid {
    frame: Frame,
    roads: CellTable,
    nodes: CellTable,
}

type CellRect = (usize, usize, usize, usize);

impl Frame {
    fn covering(points: impl Iterator<Item = Vec2>) -> Self {
        let (mut min, mut max) = bounds(points);
        if min.x > max.x {
            min = Vec2::default();
            max = Vec2::default();
        }
        let origin = min - Vec2::new(SPATIAL_CELL, SPATIAL_CELL);
        let extent = max - origin;
        Self {
            origin,
            cols: cells_along(extent.x),
            rows: cells_along(extent.y),
        }
    }

    fn count(&self) -> usize {
        self.cols * self.rows
    }

    fn column(&self, x: f64) -> usize {
        clamp_cell((x - self.origin.x) / SPATIAL_CELL, self.cols)
    }

    fn row(&self, y: f64) -> usize {
        clamp_cell((y - self.origin.y) / SPATIAL_CELL, self.rows)
    }

    fn cell_rect(&self, min: Vec2, max: Vec2) -> CellRect {
        (
            self.column(min.x),
            self.row(min.y),
            self.column(max.x),
            self.row(max.y),
        )
    }

    fn cell_of(&self, p: Vec2) -> usize {
        self.row(p.y) * self.cols + self.column(p.x)
    }
}

fn cells_along(extent: f64) -> usize {
    libm::floor(extent / SPATIAL_CELL) as usize + 2
}

fn clamp_cell(value: f64, count: usize) -> usize {
    let floored = libm::floor(value.max(0.0)) as usize;
    floored.min(count - 1)
}

fn cells(rect: CellRect, cols: usize) -> impl Iterator<Item = usize> {
    let (c0, r0, c1, r1) = rect;
    (r0..=r1).flat_map(move |r| (c0..=c1).map(move |c| r * cols + c))
}

impl CellTable {
    fn build(cell_count: usize, entries: &[(usize, u32)]) -> Self {
        let mut cell_start = vec![0u32; cell_count + 1];
        for &(cell, _) in entries {
            cell_start[cell + 1] += 1;
        }
        for i in 0..cell_count {
            cell_start[i + 1] += cell_start[i];
        }
        let mut cursor = cell_start.clone();
        let mut ids = vec![0u32; entries.len()];
        for &(cell, id) in entries {
            ids[cursor[cell] as usize] = id;
            cursor[cell] += 1;
        }
        Self { cell_start, ids }
    }

    fn cell(&self, cell: usize) -> &[u32] {
        let start = self.cell_start[cell] as usize;
        let end = self.cell_start[cell + 1] as usize;
        &self.ids[start..end]
    }
}

fn road_entries(frame: &Frame, roads: &RoadStore) -> Vec<(usize, u32)> {
    let mut last_in_cell: Vec<u32> = vec![u32::MAX; frame.count()];
    let mut entries = Vec::new();
    for road in 0..roads.count() as u32 {
        for pair in roads.points(road).windows(2) {
            let (min, max) = bounds([pair[0], pair[1]]);
            for cell in cells(frame.cell_rect(min, max), frame.cols) {
                if last_in_cell[cell] != road {
                    last_in_cell[cell] = road;
                    entries.push((cell, road));
                }
            }
        }
    }
    entries
}

impl SpatialGrid {
    pub fn build(roads: &RoadStore, nodes: &NodeStore) -> Self {
        let frame = Frame::covering(roads.points.iter().chain(&nodes.pos).copied());
        let node_entries: Vec<(usize, u32)> = nodes
            .pos
            .iter()
            .enumerate()
            .map(|(id, &p)| (frame.cell_of(p), id as u32))
            .collect();
        Self {
            roads: CellTable::build(frame.count(), &road_entries(&frame, roads)),
            nodes: CellTable::build(frame.count(), &node_entries),
            frame,
        }
    }

    pub fn roads_in_rect(&self, min: Vec2, max: Vec2) -> Vec<u32> {
        let mut result: Vec<u32> = cells(self.frame.cell_rect(min, max), self.frame.cols)
            .flat_map(|cell| self.roads.cell(cell).iter().copied())
            .collect();
        result.sort_unstable();
        result.dedup();
        result
    }

    pub fn nodes_within(&self, nodes: &NodeStore, center: Vec2, radius: f64) -> Vec<u32> {
        let (min, max) = around(center, radius);
        let mut result: Vec<u32> = cells(self.frame.cell_rect(min, max), self.frame.cols)
            .flat_map(|cell| self.nodes.cell(cell).iter().copied())
            .filter(|&node| nodes.pos[node as usize].distance(center) <= radius)
            .collect();
        result.sort_unstable();
        result
    }

    pub fn roads_near(&self, roads: &RoadStore, center: Vec2, radius: f64) -> Vec<u32> {
        let (min, max) = around(center, radius);
        let mut result = self.roads_in_rect(min, max);
        result.retain(|&road| road_touches(roads, road, center, radius));
        result
    }
}

fn road_touches(roads: &RoadStore, road: u32, center: Vec2, radius: f64) -> bool {
    roads
        .points(road)
        .iter()
        .any(|p| p.distance(center) <= radius)
}

fn around(center: Vec2, radius: f64) -> Bounds {
    let r = Vec2::new(radius, radius);
    (center - r, center + r)
}
