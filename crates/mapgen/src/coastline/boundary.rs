use super::Point;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LonLatBox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

impl LonLatBox {
    pub fn contains(&self, (lon, lat): Point) -> bool {
        (self.min_lon..=self.max_lon).contains(&lon) && (self.min_lat..=self.max_lat).contains(&lat)
    }

    fn width(&self) -> f64 {
        self.max_lon - self.min_lon
    }

    fn height(&self) -> f64 {
        self.max_lat - self.min_lat
    }

    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width() + self.height())
    }

    pub fn position(&self, edge: Edge, (lon, lat): Point) -> f64 {
        let (w, h) = (self.width(), self.height());
        let raw = match edge {
            Edge::Top => lon - self.min_lon,
            Edge::Right => w + (self.max_lat - lat),
            Edge::Bottom => w + h + (self.max_lon - lon),
            Edge::Left => self.perimeter() - (self.max_lat - lat),
        };
        raw.rem_euclid(self.perimeter())
    }

    pub fn corners(&self) -> [(f64, Point); 4] {
        let (w, h) = (self.width(), self.height());
        [
            (0.0, (self.min_lon, self.max_lat)),
            (w, (self.max_lon, self.max_lat)),
            (w + h, (self.max_lon, self.min_lat)),
            (2.0 * w + h, (self.min_lon, self.min_lat)),
        ]
    }

    pub fn snap(&self, edge: Edge, (lon, lat): Point) -> Point {
        match edge {
            Edge::Top => (lon, self.max_lat),
            Edge::Right => (self.max_lon, lat),
            Edge::Bottom => (lon, self.min_lat),
            Edge::Left => (self.min_lon, lat),
        }
    }

    pub fn edge_of(&self, (lon, lat): Point) -> Option<Edge> {
        if !self.contains((lon, lat)) {
            return None;
        }
        [
            (lat == self.max_lat, Edge::Top),
            (lon == self.max_lon, Edge::Right),
            (lat == self.min_lat, Edge::Bottom),
            (lon == self.min_lon, Edge::Left),
        ]
        .into_iter()
        .find_map(|(on, edge)| on.then_some(edge))
    }
}

#[cfg(test)]
mod tests {
    use super::{Edge, LonLatBox};

    const BOX: LonLatBox = LonLatBox {
        min_lon: 120.95,
        min_lat: 14.55,
        max_lon: 121.05,
        max_lat: 14.65,
    };

    #[test]
    fn top_left_corner_is_zero_on_both_edges() {
        let corner = (BOX.min_lon, BOX.max_lat);
        assert_eq!(BOX.position(Edge::Top, corner), 0.0);
        assert_eq!(BOX.position(Edge::Left, corner), 0.0);
    }
}
