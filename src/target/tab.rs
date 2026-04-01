use std::path::Path;
use zellij_tile::prelude::LayoutInfo;

use crate::integrations::{
    zellij_focus_or_create_tab, zellij_go_to_tab_name, zellij_hide_self,
};

use super::traits::Target;

pub struct TabTarget;

impl Target for TabTarget {
    fn create(&self, name: &str, _directory: &Path, _layout: Option<LayoutInfo>) {
        // Note: Tab API doesn't support setting cwd directly
        // The tab will be created and we rely on the user's shell to handle directory
        zellij_focus_or_create_tab(name);
    }

    fn switch_to(&self, name: &str) {
        zellij_go_to_tab_name(name);
    }

    fn delete(&self, _name: &str) -> Result<(), String> {
        Err("Tab deletion is not supported — switch to the tab and close it manually".to_string())
    }

    fn hide(&self) {
        zellij_hide_self();
    }
}
