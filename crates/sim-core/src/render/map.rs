use crate::map::MapData;

pub struct RoadRender {
    pub point_start: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub class: Vec<u8>,
    pub lanes_forward: Vec<u8>,
    pub lanes_backward: Vec<u8>,
    pub layer: Vec<i8>,
    pub name: Vec<u32>,
    pub roundabout: Vec<u8>,
    pub from: Vec<u32>,
    pub to: Vec<u32>,
}

pub struct NodeRender {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub control: Vec<u8>,
}

pub struct AreaRender {
    pub kind: Vec<u8>,
    pub ring_start: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

pub fn road_render(map: &MapData) -> RoadRender {
    let roads = &map.roads;
    RoadRender {
        point_start: roads.point_start.clone(),
        x: map.points.x.clone(),
        y: map.points.y.clone(),
        class: roads.class.iter().map(|c| c.code()).collect(),
        lanes_forward: roads.lanes_forward.clone(),
        lanes_backward: roads.lanes_backward.clone(),
        layer: roads.layer.clone(),
        name: roads.name.clone(),
        roundabout: roads
            .roundabout
            .iter()
            .map(|&flag| u8::from(flag))
            .collect(),
        from: roads.from.clone(),
        to: roads.to.clone(),
    }
}

pub fn node_render(map: &MapData) -> NodeRender {
    NodeRender {
        x: map.nodes.x.clone(),
        y: map.nodes.y.clone(),
        control: map.nodes.control.iter().map(|c| c.code()).collect(),
    }
}

pub fn area_render(map: &MapData) -> AreaRender {
    let areas = &map.areas;
    AreaRender {
        kind: areas.kind.iter().map(|k| k.code()).collect(),
        ring_start: areas.ring_start.clone(),
        x: areas.x.clone(),
        y: areas.y.clone(),
    }
}
