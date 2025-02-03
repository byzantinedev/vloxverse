use wasm_bindgen::prelude::wasm_bindgen;

mod app;

#[wasm_bindgen(start)]
pub fn start() {
    app::start();
}
