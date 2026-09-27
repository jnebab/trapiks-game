use trapiks_sim_core::geo::Projection;

use crate::areas::{AreaRings, finish_ring, project_ring};

type Point = (f64, f64);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon {
    pub outer: Vec<(f32, f32)>,
    pub holes: Vec<Vec<(f32, f32)>>,
}

pub fn finish_polygons(rings: &AreaRings, projection: &Projection) -> Vec<Polygon> {
    let inners: Vec<Vec<Point>> = rings
        .inners
        .iter()
        .map(|ring| project_ring(ring, projection))
        .collect();
    rings
        .outers
        .iter()
        .filter_map(|outer| finish_polygon(&project_ring(outer, projection), &inners))
        .collect()
}

fn finish_polygon(outer: &[Point], inners: &[Vec<Point>]) -> Option<Polygon> {
    let finished = finish_ring(outer)?;
    let holes = inners
        .iter()
        .filter(|inner| inner.first().is_some_and(|&first| contains(outer, first)))
        .filter_map(|inner| finish_ring(inner))
        .collect();
    Some(Polygon {
        outer: finished,
        holes,
    })
}

pub fn contains(ring: &[Point], point: Point) -> bool {
    let mut inside = false;
    let previous = ring.iter().cycle().skip(ring.len().saturating_sub(1));
    for (&a, &b) in ring.iter().zip(previous) {
        if crosses(a, b, point) {
            inside = !inside;
        }
    }
    inside
}

fn crosses(a: Point, b: Point, point: Point) -> bool {
    if (a.1 > point.1) == (b.1 > point.1) {
        return false;
    }
    let x = a.0 + (point.1 - a.1) / (b.1 - a.1) * (b.0 - a.0);
    point.0 < x
}
