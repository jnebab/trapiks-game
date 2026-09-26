use crate::consts::LANE_WIDTH;
use crate::geom::{Vec2, cumulative_lengths};
use crate::map::{MapData, RoadClass};

const KPH_PER_MPS: f64 = 3.6;

#[derive(Clone, Debug, Default)]
pub struct RoadStore {
    pub from: Vec<u32>,
    pub to: Vec<u32>,
    pub class: Vec<RoadClass>,
    pub lanes_forward: Vec<u8>,
    pub lanes_backward: Vec<u8>,
    pub layer: Vec<i8>,
    pub name: Vec<u32>,
    pub speed_kph: Vec<u8>,
    pub speed: Vec<f64>,
    pub roundabout: Vec<bool>,
    pub deleted: Vec<bool>,
    pub point_range: Vec<(u32, u32)>,
    pub points: Vec<Vec2>,
    pub cumulative: Vec<f64>,
    pub length: Vec<f64>,
}

impl RoadStore {
    pub fn from_map(map: &MapData) -> Self {
        let table = &map.roads;
        let mut store = RoadStore {
            from: table.from.clone(),
            to: table.to.clone(),
            class: table.class.clone(),
            lanes_forward: table.lanes_forward.clone(),
            lanes_backward: table.lanes_backward.clone(),
            layer: table.layer.clone(),
            name: table.name.clone(),
            speed_kph: table.speed_kph.clone(),
            speed: table.speed_kph.iter().map(|&kph| mps(kph)).collect(),
            roundabout: table.roundabout.clone(),
            deleted: vec![false; map.road_count()],
            ..RoadStore::default()
        };
        for road in 0..map.road_count() {
            store.push_geometry(map, road);
        }
        store
    }

    fn push_geometry(&mut self, map: &MapData, road: usize) {
        let (xs, ys) = map.road_points(road as u32);
        let points: Vec<Vec2> = xs
            .iter()
            .zip(ys)
            .map(|(&x, &y)| Vec2::new(f64::from(x), f64::from(y)))
            .collect();
        let (range, length) = self.push_points(&points);
        self.point_range.push(range);
        self.length.push(length);
    }

    pub(crate) fn push_points(&mut self, points: &[Vec2]) -> ((u32, u32), f64) {
        let cumulative = cumulative_lengths(points);
        let length = cumulative.last().copied().unwrap_or_default();
        let range = (self.points.len() as u32, points.len() as u32);
        self.points.extend_from_slice(points);
        self.cumulative.extend(cumulative);
        (range, length)
    }

    pub(crate) fn set_geometry(&mut self, road: u32, range: (u32, u32), length: f64) {
        let index = road as usize;
        self.point_range[index] = range;
        self.length[index] = length;
    }

    pub fn count(&self) -> usize {
        self.from.len()
    }

    fn range(&self, road: u32) -> std::ops::Range<usize> {
        let (start, len) = self
            .point_range
            .get(road as usize)
            .copied()
            .unwrap_or_default();
        start as usize..(start + len) as usize
    }

    pub fn points(&self, road: u32) -> &[Vec2] {
        self.points.get(self.range(road)).unwrap_or_default()
    }

    pub fn cumulative(&self, road: u32) -> &[f64] {
        self.cumulative.get(self.range(road)).unwrap_or_default()
    }

    pub fn total_lanes(&self, road: u32) -> u8 {
        let index = road as usize;
        self.lanes_forward[index].saturating_add(self.lanes_backward[index])
    }

    pub fn width(&self, road: u32) -> f64 {
        f64::from(self.total_lanes(road)) * LANE_WIDTH
    }

    pub fn set_speed_kph(&mut self, road: u32, kph: u8) {
        let index = road as usize;
        self.speed_kph[index] = kph;
        self.speed[index] = mps(kph);
    }

    pub fn is_roundabout(&self, road: u32) -> bool {
        self.roundabout.get(road as usize).copied().unwrap_or(false)
    }

    pub fn is_live(&self, road: u32) -> bool {
        !self.deleted[road as usize]
    }
}

fn mps(kph: u8) -> f64 {
    f64::from(kph) / KPH_PER_MPS
}
