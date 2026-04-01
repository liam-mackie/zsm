#[cfg(target_arch = "wasm32")]
use zsm::app::AppState;
#[cfg(target_arch = "wasm32")]
use zellij_tile::prelude::*;

#[cfg(target_arch = "wasm32")]
register_plugin!(AppState);

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("This is a Zellij plugin. Build with: cargo build --target wasm32-wasip1");
}
