use std::path::Path;
use zellij_tile::prelude::LayoutInfo;

pub trait Target {
    fn create(&self, name: &str, directory: &Path, layout: Option<LayoutInfo>);
    fn switch_to(&self, name: &str);
    fn delete(&self, name: &str) -> Result<(), String>;
    fn hide(&self);
}
