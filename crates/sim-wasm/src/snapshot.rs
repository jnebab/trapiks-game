use trapiks_sim_core::sim::Snapshot;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SnapshotPointers {
    ids: u32,
    x: u32,
    y: u32,
    heading: u32,
    style: u32,
    layer: u32,
    capacity: u32,
}

impl SnapshotPointers {
    pub fn of(snapshot: &Snapshot) -> Self {
        Self {
            ids: snapshot.ids.as_ptr() as u32,
            x: snapshot.x.as_ptr() as u32,
            y: snapshot.y.as_ptr() as u32,
            heading: snapshot.heading.as_ptr() as u32,
            style: snapshot.style.as_ptr() as u32,
            layer: snapshot.layer.as_ptr() as u32,
            capacity: snapshot.ids.capacity() as u32,
        }
    }
}

#[wasm_bindgen]
impl SnapshotPointers {
    #[wasm_bindgen(getter)]
    pub fn ids(&self) -> u32 {
        self.ids
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[wasm_bindgen(getter)]
    pub fn heading(&self) -> u32 {
        self.heading
    }

    #[wasm_bindgen(getter)]
    pub fn style(&self) -> u32 {
        self.style
    }

    #[wasm_bindgen(getter)]
    pub fn layer(&self) -> u32 {
        self.layer
    }

    #[wasm_bindgen(getter)]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}
