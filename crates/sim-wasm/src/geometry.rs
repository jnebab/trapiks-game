use trapiks_sim_core::render::{AreaRender, NodeRender, RoadRender};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RoadGeometry(RoadRender);

#[wasm_bindgen]
pub struct NodeGeometry(NodeRender);

#[wasm_bindgen]
pub struct AreaGeometry(AreaRender);

impl From<RoadRender> for RoadGeometry {
    fn from(render: RoadRender) -> Self {
        Self(render)
    }
}

impl From<NodeRender> for NodeGeometry {
    fn from(render: NodeRender) -> Self {
        Self(render)
    }
}

impl From<AreaRender> for AreaGeometry {
    fn from(render: AreaRender) -> Self {
        Self(render)
    }
}

#[wasm_bindgen]
impl RoadGeometry {
    #[wasm_bindgen(getter, js_name = pointStart)]
    pub fn point_start(&self) -> Vec<u32> {
        self.0.point_start.clone()
    }

    #[wasm_bindgen(getter, js_name = x)]
    pub fn x(&self) -> Vec<f32> {
        self.0.x.clone()
    }

    #[wasm_bindgen(getter, js_name = y)]
    pub fn y(&self) -> Vec<f32> {
        self.0.y.clone()
    }

    #[wasm_bindgen(getter, js_name = classCode)]
    pub fn class_code(&self) -> Vec<u8> {
        self.0.class.clone()
    }

    #[wasm_bindgen(getter, js_name = lanesForward)]
    pub fn lanes_forward(&self) -> Vec<u8> {
        self.0.lanes_forward.clone()
    }

    #[wasm_bindgen(getter, js_name = lanesBackward)]
    pub fn lanes_backward(&self) -> Vec<u8> {
        self.0.lanes_backward.clone()
    }

    #[wasm_bindgen(getter, js_name = layer)]
    pub fn layer(&self) -> Vec<i8> {
        self.0.layer.clone()
    }

    #[wasm_bindgen(getter, js_name = name)]
    pub fn name(&self) -> Vec<u32> {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter, js_name = from)]
    pub fn from_node(&self) -> Vec<u32> {
        self.0.from.clone()
    }

    #[wasm_bindgen(getter, js_name = to)]
    pub fn to_node(&self) -> Vec<u32> {
        self.0.to.clone()
    }
}

#[wasm_bindgen]
impl NodeGeometry {
    #[wasm_bindgen(getter, js_name = x)]
    pub fn x(&self) -> Vec<f32> {
        self.0.x.clone()
    }

    #[wasm_bindgen(getter, js_name = y)]
    pub fn y(&self) -> Vec<f32> {
        self.0.y.clone()
    }

    #[wasm_bindgen(getter, js_name = controlCode)]
    pub fn control_code(&self) -> Vec<u8> {
        self.0.control.clone()
    }
}

#[wasm_bindgen]
impl AreaGeometry {
    #[wasm_bindgen(getter, js_name = kindCode)]
    pub fn kind_code(&self) -> Vec<u8> {
        self.0.kind.clone()
    }

    #[wasm_bindgen(getter, js_name = ringStart)]
    pub fn ring_start(&self) -> Vec<u32> {
        self.0.ring_start.clone()
    }

    #[wasm_bindgen(getter, js_name = x)]
    pub fn x(&self) -> Vec<f32> {
        self.0.x.clone()
    }

    #[wasm_bindgen(getter, js_name = y)]
    pub fn y(&self) -> Vec<f32> {
        self.0.y.clone()
    }
}
