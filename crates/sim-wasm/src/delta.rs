use trapiks_sim_core::render::NetworkDelta;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DeltaGeometry(NetworkDelta);

impl From<NetworkDelta> for DeltaGeometry {
    fn from(delta: NetworkDelta) -> Self {
        Self(delta)
    }
}

#[wasm_bindgen]
impl DeltaGeometry {
    #[wasm_bindgen(getter, js_name = roadCount)]
    pub fn road_count(&self) -> u32 {
        self.0.road_count
    }

    #[wasm_bindgen(getter, js_name = nodeCount)]
    pub fn node_count(&self) -> u32 {
        self.0.node_count
    }

    #[wasm_bindgen(getter, js_name = roadIds)]
    pub fn road_ids(&self) -> Vec<u32> {
        self.0.roads.id.clone()
    }

    #[wasm_bindgen(getter, js_name = roadClass)]
    pub fn road_class(&self) -> Vec<u8> {
        self.0.roads.class.clone()
    }

    #[wasm_bindgen(getter, js_name = roadLanesForward)]
    pub fn road_lanes_forward(&self) -> Vec<u8> {
        self.0.roads.lanes_forward.clone()
    }

    #[wasm_bindgen(getter, js_name = roadLanesBackward)]
    pub fn road_lanes_backward(&self) -> Vec<u8> {
        self.0.roads.lanes_backward.clone()
    }

    #[wasm_bindgen(getter, js_name = roadLayer)]
    pub fn road_layer(&self) -> Vec<i8> {
        self.0.roads.layer.clone()
    }

    #[wasm_bindgen(getter, js_name = roadName)]
    pub fn road_name(&self) -> Vec<u32> {
        self.0.roads.name.clone()
    }

    #[wasm_bindgen(getter, js_name = roadRoundabout)]
    pub fn road_roundabout(&self) -> Vec<u8> {
        self.0.roads.roundabout.clone()
    }

    #[wasm_bindgen(getter, js_name = roadDeleted)]
    pub fn road_deleted(&self) -> Vec<u8> {
        self.0.roads.deleted.clone()
    }

    #[wasm_bindgen(getter, js_name = roadFrom)]
    pub fn road_from(&self) -> Vec<u32> {
        self.0.roads.from.clone()
    }

    #[wasm_bindgen(getter, js_name = roadTo)]
    pub fn road_to(&self) -> Vec<u32> {
        self.0.roads.to.clone()
    }

    #[wasm_bindgen(getter, js_name = roadPointLen)]
    pub fn road_point_len(&self) -> Vec<u32> {
        self.0.roads.point_len.clone()
    }

    #[wasm_bindgen(getter, js_name = roadX)]
    pub fn road_x(&self) -> Vec<f32> {
        self.0.roads.x.clone()
    }

    #[wasm_bindgen(getter, js_name = roadY)]
    pub fn road_y(&self) -> Vec<f32> {
        self.0.roads.y.clone()
    }

    #[wasm_bindgen(getter, js_name = nodeIds)]
    pub fn node_ids(&self) -> Vec<u32> {
        self.0.nodes.id.clone()
    }

    #[wasm_bindgen(getter, js_name = nodeX)]
    pub fn node_x(&self) -> Vec<f32> {
        self.0.nodes.x.clone()
    }

    #[wasm_bindgen(getter, js_name = nodeY)]
    pub fn node_y(&self) -> Vec<f32> {
        self.0.nodes.y.clone()
    }

    #[wasm_bindgen(getter, js_name = nodeControl)]
    pub fn node_control(&self) -> Vec<u8> {
        self.0.nodes.control.clone()
    }

    #[wasm_bindgen(getter, js_name = setbackIds)]
    pub fn setback_ids(&self) -> Vec<u32> {
        self.0.setbacks.id.clone()
    }

    #[wasm_bindgen(getter, js_name = setbackStart)]
    pub fn setback_start(&self) -> Vec<f32> {
        self.0.setbacks.start.clone()
    }

    #[wasm_bindgen(getter, js_name = setbackEnd)]
    pub fn setback_end(&self) -> Vec<f32> {
        self.0.setbacks.end.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionNode)]
    pub fn junction_node(&self) -> Vec<u32> {
        self.0.junctions.node.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionLayer)]
    pub fn junction_layer(&self) -> Vec<i8> {
        self.0.junctions.layer.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionMinLayer)]
    pub fn junction_min_layer(&self) -> Vec<i8> {
        self.0.junctions.min_layer.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionRingStart)]
    pub fn junction_ring_start(&self) -> Vec<u32> {
        self.0.junctions.ring_start.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionX)]
    pub fn junction_x(&self) -> Vec<f32> {
        self.0.junctions.x.clone()
    }

    #[wasm_bindgen(getter, js_name = junctionY)]
    pub fn junction_y(&self) -> Vec<f32> {
        self.0.junctions.y.clone()
    }

    #[wasm_bindgen(getter, js_name = markerNodes)]
    pub fn marker_nodes(&self) -> Vec<u32> {
        self.0.markers.nodes.clone()
    }

    #[wasm_bindgen(getter, js_name = markerLink)]
    pub fn marker_link(&self) -> Vec<u32> {
        self.0.markers.markers.link.clone()
    }

    #[wasm_bindgen(getter, js_name = markerNode)]
    pub fn marker_node(&self) -> Vec<u32> {
        self.0.markers.markers.node.clone()
    }

    #[wasm_bindgen(getter, js_name = markerKind)]
    pub fn marker_kind(&self) -> Vec<u8> {
        self.0.markers.markers.kind.clone()
    }

    #[wasm_bindgen(getter, js_name = markerX1)]
    pub fn marker_x1(&self) -> Vec<f32> {
        self.0.markers.markers.x1.clone()
    }

    #[wasm_bindgen(getter, js_name = markerY1)]
    pub fn marker_y1(&self) -> Vec<f32> {
        self.0.markers.markers.y1.clone()
    }

    #[wasm_bindgen(getter, js_name = markerX2)]
    pub fn marker_x2(&self) -> Vec<f32> {
        self.0.markers.markers.x2.clone()
    }

    #[wasm_bindgen(getter, js_name = markerY2)]
    pub fn marker_y2(&self) -> Vec<f32> {
        self.0.markers.markers.y2.clone()
    }

    #[wasm_bindgen(getter, js_name = pillLink)]
    pub fn pill_link(&self) -> Vec<u32> {
        self.0.pills.link.clone()
    }

    #[wasm_bindgen(getter, js_name = pillX)]
    pub fn pill_x(&self) -> Vec<f32> {
        self.0.pills.x.clone()
    }

    #[wasm_bindgen(getter, js_name = pillY)]
    pub fn pill_y(&self) -> Vec<f32> {
        self.0.pills.y.clone()
    }

    #[wasm_bindgen(getter, js_name = pillAngle)]
    pub fn pill_angle(&self) -> Vec<f32> {
        self.0.pills.angle.clone()
    }
}
