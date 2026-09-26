use trapiks_sim_core::render::SignalPills;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SignalPillGeometry(SignalPills);

impl From<SignalPills> for SignalPillGeometry {
    fn from(pills: SignalPills) -> Self {
        Self(pills)
    }
}

#[wasm_bindgen]
impl SignalPillGeometry {
    #[wasm_bindgen(getter)]
    pub fn link(&self) -> Vec<u32> {
        self.0.link.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> Vec<f32> {
        self.0.x.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> Vec<f32> {
        self.0.y.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn angle(&self) -> Vec<f32> {
        self.0.angle.clone()
    }
}
