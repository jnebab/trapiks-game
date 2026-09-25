use std::collections::BTreeMap;

use crate::map::{AreaKind, Control, GeoOrigin, MapData, RoadClass, TurnBan};

const ORIGIN_LAT: f64 = 14.5995;
const ORIGIN_LON: f64 = 120.9842;

#[derive(Clone, Debug, PartialEq)]
pub struct RoadSpec<'a> {
    pub class: RoadClass,
    pub lanes_forward: u8,
    pub lanes_backward: u8,
    pub speed_kph: u8,
    pub layer: i8,
    pub name: &'a str,
    pub via: Vec<(f32, f32)>,
}

impl<'a> RoadSpec<'a> {
    pub fn new(class: RoadClass, forward: u8, backward: u8) -> Self {
        Self {
            class,
            lanes_forward: forward,
            lanes_backward: backward,
            speed_kph: class.default_speed_kph(),
            layer: 0,
            name: "",
            via: Vec::new(),
        }
    }

    pub fn speed(mut self, kph: u8) -> Self {
        self.speed_kph = kph;
        self
    }

    pub fn layer(mut self, layer: i8) -> Self {
        self.layer = layer;
        self
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn via(mut self, via: Vec<(f32, f32)>) -> Self {
        self.via = via;
        self
    }
}

type Ring = (AreaKind, Vec<(f32, f32)>);

pub struct MapBuilder {
    map: MapData,
    name_lookup: BTreeMap<String, u32>,
    rings: Vec<Ring>,
}

impl Default for MapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MapBuilder {
    pub fn new() -> Self {
        let mut map = MapData {
            origin: GeoOrigin {
                lat: ORIGIN_LAT,
                lon: ORIGIN_LON,
            },
            names: vec![String::new()],
            ..MapData::default()
        };
        map.roads.point_start.push(0);
        Self {
            map,
            name_lookup: BTreeMap::from([(String::new(), 0)]),
            rings: Vec::new(),
        }
    }

    pub fn node(&mut self, x: f32, y: f32) -> u32 {
        let id = to_u32(self.map.node_count());
        self.map.nodes.x.push(x);
        self.map.nodes.y.push(y);
        self.map.nodes.control.push(Control::Priority);
        id
    }

    pub fn road(&mut self, from: u32, to: u32, spec: RoadSpec) -> u32 {
        let id = to_u32(self.map.road_count());
        let name = self.name_index(spec.name);
        let roads = &mut self.map.roads;
        roads.from.push(from);
        roads.to.push(to);
        roads.class.push(spec.class);
        roads.lanes_forward.push(spec.lanes_forward);
        roads.lanes_backward.push(spec.lanes_backward);
        roads.speed_kph.push(spec.speed_kph);
        roads.layer.push(spec.layer);
        roads.name.push(name);
        self.push_points(from, to, &spec.via);
        id
    }

    pub fn control(&mut self, node: u32, control: Control) {
        if let Some(slot) = self.map.nodes.control.get_mut(node as usize) {
            *slot = control;
        }
    }

    pub fn ban(&mut self, from_road: u32, via_node: u32, to_road: u32) {
        self.map.turn_bans.push(TurnBan {
            via_node,
            from_road,
            to_road,
        });
    }

    pub fn area(&mut self, kind: AreaKind, points: &[(f32, f32)]) {
        self.rings.push((kind, points.to_vec()));
    }

    pub fn build(mut self) -> MapData {
        self.map.turn_bans.sort_unstable();
        self.map.turn_bans.dedup();
        self.rings.sort_by_key(|(kind, _)| *kind == AreaKind::Park);
        self.map.areas.ring_start.push(0);
        for (kind, points) in &self.rings {
            push_ring(&mut self.map, *kind, points);
        }
        self.map
    }

    fn name_index(&mut self, name: &str) -> u32 {
        if let Some(&index) = self.name_lookup.get(name) {
            return index;
        }
        let index = to_u32(self.map.names.len());
        self.map.names.push(name.to_string());
        self.name_lookup.insert(name.to_string(), index);
        index
    }

    fn node_position(&self, node: u32) -> (f32, f32) {
        let index = node as usize;
        let x = self.map.nodes.x.get(index).copied().unwrap_or_default();
        let y = self.map.nodes.y.get(index).copied().unwrap_or_default();
        (x, y)
    }

    fn push_points(&mut self, from: u32, to: u32, via: &[(f32, f32)]) {
        let start = self.node_position(from);
        let end = self.node_position(to);
        let points = &mut self.map.points;
        for (x, y) in std::iter::once(start)
            .chain(via.iter().copied())
            .chain([end])
        {
            points.x.push(x);
            points.y.push(y);
        }
        let count = to_u32(points.x.len());
        self.map.roads.point_start.push(count);
    }
}

fn push_ring(map: &mut MapData, kind: AreaKind, points: &[(f32, f32)]) {
    let areas = &mut map.areas;
    areas.kind.push(kind);
    for (x, y) in points {
        areas.x.push(*x);
        areas.y.push(*y);
    }
    areas.ring_start.push(to_u32(areas.x.len()));
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
