use super::Point;
use super::boundary::LonLatBox;
use super::clip::Piece;

struct Walk<'a> {
    pieces: &'a [Piece],
    bbox: &'a LonLatBox,
    entry_t: Vec<f64>,
    used: Vec<bool>,
}

pub fn sea_polygons(pieces: &[Piece], bbox: &LonLatBox) -> (Vec<Vec<Point>>, u32) {
    let mut walk = Walk::new(pieces, bbox);
    let mut order: Vec<usize> = (0..pieces.len()).collect();
    order.sort_by(|&a, &b| walk.entry_t[a].total_cmp(&walk.entry_t[b]));
    let mut rings = Vec::new();
    let mut dropped = 0;
    for start in order {
        if walk.used[start] {
            continue;
        }
        match walk.cycle(start) {
            Ok(ring) => rings.push(ring),
            Err(count) => dropped += count,
        }
    }
    (rings, dropped)
}

impl<'a> Walk<'a> {
    fn new(pieces: &'a [Piece], bbox: &'a LonLatBox) -> Self {
        let entry_t = pieces
            .iter()
            .map(|p| bbox.position(p.entry_edge, p.points[0]))
            .collect();
        Self {
            pieces,
            bbox,
            entry_t,
            used: vec![false; pieces.len()],
        }
    }

    fn cycle(&mut self, start: usize) -> Result<Vec<Point>, u32> {
        let mut ring = Vec::new();
        let mut current = start;
        let mut followed = 0;
        loop {
            self.used[current] = true;
            followed += 1;
            let piece = &self.pieces[current];
            ring.extend_from_slice(&piece.points);
            let exit = piece.points[piece.points.len() - 1];
            let exit_t = self.bbox.position(piece.exit_edge, exit);
            let next = self.next_entry(exit_t);
            ring.extend(self.corners_between(exit_t, self.entry_t[next]));
            if next == start {
                return Ok(ring);
            }
            if self.used[next] {
                return Err(followed);
            }
            current = next;
        }
    }

    fn clockwise(&self, from: f64, to: f64) -> f64 {
        (to - from).rem_euclid(self.bbox.perimeter())
    }

    fn next_entry(&self, exit_t: f64) -> usize {
        let mut best = 0;
        for index in 1..self.pieces.len() {
            let distance = self.clockwise(exit_t, self.entry_t[index]);
            if distance < self.clockwise(exit_t, self.entry_t[best]) {
                best = index;
            }
        }
        best
    }

    fn corners_between(&self, from: f64, to: f64) -> Vec<Point> {
        let span = self.clockwise(from, to);
        let mut corners: Vec<(f64, Point)> = self
            .bbox
            .corners()
            .into_iter()
            .map(|(t, point)| (self.clockwise(from, t), point))
            .filter(|&(distance, _)| distance > 0.0 && distance < span)
            .collect();
        corners.sort_by(|a, b| a.0.total_cmp(&b.0));
        corners.into_iter().map(|(_, point)| point).collect()
    }
}
