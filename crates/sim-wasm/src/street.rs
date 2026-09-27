use trapiks_sim_core::render::{ApproachMarkers, JunctionShapes};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JunctionShapeGeometry(JunctionShapes);

#[wasm_bindgen]
pub struct ApproachMarkerGeometry(ApproachMarkers);

impl From<JunctionShapes> for JunctionShapeGeometry {
    fn from(shapes: JunctionShapes) -> Self {
        Self(shapes)
    }
}

impl From<ApproachMarkers> for ApproachMarkerGeometry {
    fn from(markers: ApproachMarkers) -> Self {
        Self(markers)
    }
}

#[wasm_bindgen]
impl JunctionShapeGeometry {
    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Vec<u32> {
        self.0.node.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn layer(&self) -> Vec<i8> {
        self.0.layer.clone()
    }

    #[wasm_bindgen(getter, js_name = minLayer)]
    pub fn min_layer(&self) -> Vec<i8> {
        self.0.min_layer.clone()
    }

    #[wasm_bindgen(getter, js_name = filletLayer)]
    pub fn fillet_layer(&self) -> Vec<i8> {
        self.0.fillet_layer.clone()
    }

    #[wasm_bindgen(getter, js_name = ringStart)]
    pub fn ring_start(&self) -> Vec<u32> {
        self.0.ring_start.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> Vec<f32> {
        self.0.x.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> Vec<f32> {
        self.0.y.clone()
    }
}

#[wasm_bindgen]
impl ApproachMarkerGeometry {
    #[wasm_bindgen(getter)]
    pub fn link(&self) -> Vec<u32> {
        self.0.link.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Vec<u32> {
        self.0.node.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> Vec<u8> {
        self.0.kind.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn x1(&self) -> Vec<f32> {
        self.0.x1.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y1(&self) -> Vec<f32> {
        self.0.y1.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn x2(&self) -> Vec<f32> {
        self.0.x2.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y2(&self) -> Vec<f32> {
        self.0.y2.clone()
    }
}
