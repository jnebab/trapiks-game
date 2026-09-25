use crate::geom::Vec2;
use crate::map::{Control, MapData};

#[derive(Clone, Debug, Default)]
pub struct NodeStore {
    pub pos: Vec<Vec2>,
    pub control: Vec<Control>,
    pub roads: Vec<Vec<u32>>,
}

impl NodeStore {
    pub fn from_map(map: &MapData) -> Self {
        let nodes = &map.nodes;
        let pos = nodes
            .x
            .iter()
            .zip(&nodes.y)
            .map(|(&x, &y)| Vec2::new(f64::from(x), f64::from(y)))
            .collect();
        let mut roads = vec![Vec::new(); map.node_count()];
        for (road, (&from, &to)) in map.roads.from.iter().zip(&map.roads.to).enumerate() {
            attach(&mut roads, from, road as u32);
            if to != from {
                attach(&mut roads, to, road as u32);
            }
        }
        Self {
            pos,
            control: nodes.control.clone(),
            roads,
        }
    }

    pub fn count(&self) -> usize {
        self.pos.len()
    }

    pub fn active_degree(&self, node: u32, is_active: impl Fn(u32) -> bool) -> usize {
        self.roads[node as usize]
            .iter()
            .filter(|&&road| is_active(road))
            .count()
    }
}

fn attach(roads: &mut [Vec<u32>], node: u32, road: u32) {
    if let Some(list) = roads.get_mut(node as usize) {
        list.push(road);
    }
}
