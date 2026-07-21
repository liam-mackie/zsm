pub mod app;
pub mod data;
pub mod domain;
pub mod input;
pub mod integrations;
pub mod naming;
pub mod search;
pub mod target;
pub mod ui;

#[cfg(target_arch = "wasm32")]
mod plugin;
