use std::collections::BTreeMap;
use std::path::PathBuf;

#[cfg(not(test))]
use zellij_tile::prelude::{
    close_focused_tab, delete_dead_session, focus_or_create_tab, go_to_tab_name, hide_self,
    kill_sessions, pipe_message_to_plugin, run_command, switch_session, switch_session_with_cwd,
    switch_session_with_layout, LayoutInfo, MessageToPlugin,
};

#[cfg(test)]
use zellij_tile::prelude::LayoutInfo;

// Real implementations

#[cfg(not(test))]
pub fn zellij_hide_self() {
    hide_self();
}

#[cfg(not(test))]
pub fn zellij_switch_session(name: Option<&str>) {
    switch_session(name);
}

#[cfg(not(test))]
pub fn zellij_switch_session_with_cwd(name: Option<&str>, cwd: Option<PathBuf>) {
    switch_session_with_cwd(name, cwd);
}

#[cfg(not(test))]
pub fn zellij_switch_session_with_layout(
    name: Option<&str>,
    layout: LayoutInfo,
    cwd: Option<PathBuf>,
) {
    switch_session_with_layout(name, layout, cwd);
}

#[cfg(not(test))]
pub fn zellij_kill_sessions(names: &[String]) {
    kill_sessions(names);
}

#[cfg(not(test))]
pub fn zellij_delete_dead_session(name: &str) {
    delete_dead_session(name);
}

#[cfg(not(test))]
pub fn zellij_focus_or_create_tab(name: &str) {
    focus_or_create_tab(name);
}

#[cfg(not(test))]
pub fn zellij_go_to_tab_name(name: &str) {
    go_to_tab_name(name);
}

#[cfg(not(test))]
pub fn zellij_close_focused_tab() {
    close_focused_tab();
}

#[cfg(not(test))]
pub fn zellij_pipe_message_to_plugin(
    plugin_name: &str,
    plugin_url: &str,
    config: BTreeMap<String, String>,
    args: BTreeMap<String, String>,
    pane_title: &str,
) {
    pipe_message_to_plugin(
        MessageToPlugin::new(plugin_name)
            .with_plugin_url(plugin_url)
            .with_plugin_config(config)
            .new_plugin_instance_should_have_pane_title(pane_title)
            .with_args(args),
    );
}

#[cfg(not(test))]
pub fn zellij_run_command(cmd: &[&str], context: BTreeMap<String, String>) {
    run_command(cmd, context);
}

// Test stubs

#[cfg(test)]
pub fn zellij_hide_self() {}

#[cfg(test)]
pub fn zellij_switch_session(_name: Option<&str>) {}

#[cfg(test)]
pub fn zellij_switch_session_with_cwd(_name: Option<&str>, _cwd: Option<PathBuf>) {}

#[cfg(test)]
pub fn zellij_switch_session_with_layout(
    _name: Option<&str>,
    _layout: LayoutInfo,
    _cwd: Option<PathBuf>,
) {
}

#[cfg(test)]
pub fn zellij_kill_sessions(_names: &[String]) {}

#[cfg(test)]
pub fn zellij_delete_dead_session(_name: &str) {}

#[cfg(test)]
pub fn zellij_focus_or_create_tab(_name: &str) {}

#[cfg(test)]
pub fn zellij_go_to_tab_name(_name: &str) {}

#[cfg(test)]
pub fn zellij_close_focused_tab() {}

#[cfg(test)]
pub fn zellij_pipe_message_to_plugin(
    _plugin_name: &str,
    _plugin_url: &str,
    _config: BTreeMap<String, String>,
    _args: BTreeMap<String, String>,
    _pane_title: &str,
) {
}

#[cfg(test)]
pub fn zellij_run_command(_cmd: &[&str], _context: BTreeMap<String, String>) {}
