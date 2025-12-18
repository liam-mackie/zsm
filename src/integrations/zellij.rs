use std::collections::BTreeMap;
use std::path::PathBuf;

#[cfg(not(test))]
use zellij_tile::prelude::{
    close_focused_tab, delete_dead_session, focus_or_create_tab, go_to_tab_name, hide_self,
    kill_sessions, pipe_message_to_plugin, switch_session, switch_session_with_cwd,
    switch_session_with_layout, LayoutInfo, MessageToPlugin,
};

#[cfg(test)]
use zellij_tile::prelude::LayoutInfo;

pub fn zellij_hide_self() {
    #[cfg(not(test))]
    hide_self();
}

pub fn zellij_switch_session(name: Option<&str>) {
    #[cfg(not(test))]
    switch_session(name);
    #[cfg(test)]
    let _ = name;
}

pub fn zellij_switch_session_with_cwd(name: Option<&str>, cwd: Option<PathBuf>) {
    #[cfg(not(test))]
    switch_session_with_cwd(name, cwd);
    #[cfg(test)]
    let _ = (name, cwd);
}

pub fn zellij_switch_session_with_layout(
    name: Option<&str>,
    layout: LayoutInfo,
    cwd: Option<PathBuf>,
) {
    #[cfg(not(test))]
    switch_session_with_layout(name, layout, cwd);
    #[cfg(test)]
    let _ = (name, layout, cwd);
}

pub fn zellij_kill_sessions(names: &[String]) {
    #[cfg(not(test))]
    kill_sessions(names);
    #[cfg(test)]
    let _ = names;
}

pub fn zellij_delete_dead_session(name: &str) {
    #[cfg(not(test))]
    delete_dead_session(name);
    #[cfg(test)]
    let _ = name;
}

pub fn zellij_focus_or_create_tab(name: &str) {
    #[cfg(not(test))]
    focus_or_create_tab(name);
    #[cfg(test)]
    let _ = name;
}

pub fn zellij_go_to_tab_name(name: &str) {
    #[cfg(not(test))]
    go_to_tab_name(name);
    #[cfg(test)]
    let _ = name;
}

pub fn zellij_close_focused_tab() {
    #[cfg(not(test))]
    close_focused_tab();
}

pub fn zellij_pipe_message_to_plugin(
    plugin_name: &str,
    plugin_url: &str,
    config: BTreeMap<String, String>,
    args: BTreeMap<String, String>,
    pane_title: &str,
) {
    #[cfg(not(test))]
    {
        pipe_message_to_plugin(
            MessageToPlugin::new(plugin_name)
                .with_plugin_url(plugin_url)
                .with_plugin_config(config)
                .new_plugin_instance_should_have_pane_title(pane_title)
                .with_args(args),
        );
    }
    #[cfg(test)]
    let _ = (plugin_name, plugin_url, config, args, pane_title);
}

pub fn zellij_run_command(cmd: &[&str], context: BTreeMap<String, String>) {
    #[cfg(not(test))]
    {
        use zellij_tile::prelude::run_command;
        run_command(cmd, context);
    }
    #[cfg(test)]
    let _ = (cmd, context);
}
