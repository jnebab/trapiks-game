use crate::map::{AreaKind, Control, MapData, RoadClass};

use super::{MapBuilder, RoadSpec};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridCity {
    pub cols: u32,
    pub rows: u32,
    pub spacing: f32,
}

const ARTERIAL_EVERY: u32 = 5;
const RIVER_HALF_WIDTH: f32 = 15.0;
const PARK_INSET: f32 = 10.0;

struct Grid<'a> {
    spec: &'a GridCity,
    builder: MapBuilder,
    nodes: Vec<u32>,
}

pub fn grid_city(spec: &GridCity) -> MapData {
    let mut grid = Grid::new(spec);
    grid.add_columns();
    grid.add_rows();
    grid.add_signals();
    grid.add_skyway();
    grid.add_areas();
    grid.builder.build()
}

fn is_arterial(line: u32) -> bool {
    line.is_multiple_of(ARTERIAL_EVERY)
}

fn column_spec(line: u32, name: &str) -> RoadSpec<'_> {
    if line == 2 || line == 3 {
        return RoadSpec::new(RoadClass::Secondary, 2, 0).name(name);
    }
    line_spec(line, name)
}

fn line_spec(line: u32, name: &str) -> RoadSpec<'_> {
    if is_arterial(line) {
        return RoadSpec::new(RoadClass::Primary, 2, 2).speed(60).name(name);
    }
    RoadSpec::new(RoadClass::Residential, 1, 1)
        .speed(20)
        .name(name)
}

fn column_name(line: u32) -> String {
    if is_arterial(line) {
        return format!("Avenue {line}");
    }
    format!("Street {line}")
}

fn row_name(line: u32) -> String {
    if is_arterial(line) {
        return format!("Boulevard {line}");
    }
    format!("Road {line}")
}

fn ramp() -> RoadSpec<'static> {
    RoadSpec::new(RoadClass::MotorwayLink, 1, 0).layer(1)
}

impl<'a> Grid<'a> {
    fn new(spec: &'a GridCity) -> Self {
        let mut builder = MapBuilder::new();
        let mut nodes = Vec::new();
        for i in 0..=spec.cols {
            for j in 0..=spec.rows {
                nodes.push(builder.node(i as f32 * spec.spacing, j as f32 * spec.spacing));
            }
        }
        Self {
            spec,
            builder,
            nodes,
        }
    }

    fn at(&self, i: u32, j: u32) -> u32 {
        let index = i as usize * (self.spec.rows as usize + 1) + j as usize;
        self.nodes.get(index).copied().unwrap_or_default()
    }

    fn add_columns(&mut self) {
        for i in 0..=self.spec.cols {
            let name = column_name(i);
            for j in 0..self.spec.rows {
                let (from, to) = (self.at(i, j), self.at(i, j + 1));
                let (from, to) = if i == 3 { (to, from) } else { (from, to) };
                self.builder.road(from, to, column_spec(i, &name));
            }
        }
    }

    fn add_rows(&mut self) {
        for j in 0..=self.spec.rows {
            let name = row_name(j);
            for i in 0..self.spec.cols {
                let (from, to) = (self.at(i, j), self.at(i + 1, j));
                self.builder.road(from, to, line_spec(j, &name));
            }
        }
    }

    fn add_signals(&mut self) {
        for i in (0..=self.spec.cols).filter(|&i| is_arterial(i)) {
            for j in (0..=self.spec.rows).filter(|&j| is_arterial(j)) {
                let node = self.at(i, j);
                self.builder.control(node, Control::Signal);
            }
        }
    }

    fn add_skyway(&mut self) {
        let s = self.spec.spacing;
        let (cols, rows) = (self.spec.cols as f32, self.spec.rows as f32);
        let start = (-s, -s / 2.0);
        let end = ((cols + 1.0) * s, (rows + 1.0) * s - s / 2.0);
        let start_node = self.builder.node(start.0, start.1);
        let middle_node = self
            .builder
            .node((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0);
        let end_node = self.builder.node(end.0, end.1);
        let skyway = RoadSpec::new(RoadClass::Motorway, 3, 0)
            .speed(100)
            .layer(1)
            .name("Skyway");
        self.builder.road(start_node, middle_node, skyway.clone());
        self.builder.road(middle_node, end_node, skyway);
        let (half_cols, half_rows) = (self.spec.cols / 2, self.spec.rows / 2);
        let exit = self.at(half_cols + 1, half_rows);
        let entry = self.at(half_cols, half_rows + 1);
        self.builder.road(middle_node, exit, ramp());
        self.builder.road(entry, middle_node, ramp());
    }

    fn add_areas(&mut self) {
        let s = self.spec.spacing;
        let (cols, rows) = (self.spec.cols as f32, self.spec.rows as f32);
        let river_y = 1.5 * s;
        let river = rectangle(
            (-s, river_y - RIVER_HALF_WIDTH),
            ((cols + 1.0) * s, river_y + RIVER_HALF_WIDTH),
        );
        let park = rectangle(
            ((cols - 2.0) * s + PARK_INSET, (rows - 2.0) * s + PARK_INSET),
            ((cols - 1.0) * s - PARK_INSET, (rows - 1.0) * s - PARK_INSET),
        );
        self.builder.area(AreaKind::Water, &river);
        self.builder.area(AreaKind::Park, &park);
    }
}

fn rectangle(min: (f32, f32), max: (f32, f32)) -> [(f32, f32); 4] {
    [
        (min.0, min.1),
        (max.0, min.1),
        (max.0, max.1),
        (min.0, max.1),
    ]
}
