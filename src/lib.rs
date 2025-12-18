pub mod app;
pub mod data;
pub mod domain;
pub mod input;
pub mod integrations;
pub mod naming;
pub mod search;
pub mod target;
pub mod ui;

#[cfg(target_family = "wasm")]
mod plugin;

#[cfg(target_family = "wasm")]
pub use plugin::*;
