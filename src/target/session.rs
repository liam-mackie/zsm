use std::path::Path;
use zellij_tile::prelude::LayoutInfo;

use crate::integrations::{
    zellij_hide_self, zellij_kill_sessions, zellij_switch_session,
    zellij_switch_session_with_cwd, zellij_switch_session_with_layout,
};

use super::traits::Target;

pub struct SessionTarget;

impl Target for SessionTarget {
    fn create(&self, name: &str, directory: &Path, layout: Option<LayoutInfo>) {
        let cwd = Some(directory.to_path_buf());
        match layout {
            Some(l) => zellij_switch_session_with_layout(Some(name), l, cwd),
            None => zellij_switch_session_with_cwd(Some(name), cwd),
        }
    }

    fn switch_to(&self, name: &str) {
        zellij_switch_session(Some(name));
    }

    fn delete(&self, name: &str) {
        zellij_kill_sessions(&[name.to_string()]);
    }

    fn hide(&self) {
        zellij_hide_self();
    }
}
