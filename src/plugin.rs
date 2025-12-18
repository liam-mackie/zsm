use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::AppState;
use crate::integrations::parse_zoxide_output;
use crate::ui;
use zellij_tile::prelude::*;

register_plugin!(AppState);

impl ZellijPlugin for AppState {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.initialize(configuration);

        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);

        subscribe(&[
            EventType::ModeUpdate,
            EventType::SessionUpdate,
            EventType::Key,
            EventType::RunCommandResult,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(mode_info) => {
                self.set_colors(mode_info.style.colors.into());
                true
            }
            Event::Key(key) => self.handle_key(key),
            Event::PermissionRequestResult(status) => {
                match status {
                    PermissionStatus::Granted => {
                        fetch_zoxide_directories();
                    }
                    PermissionStatus::Denied => {
                        self.set_error("Permissions denied".to_string());
                    }
                }
                true
            }
            Event::SessionUpdate(infos, resurrectable) => {
                self.update_sessions(infos);
                let durations: Vec<_> = resurrectable.into_iter().collect();
                for (name, duration) in durations {
                    let _ = (name, duration);
                }
                true
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                if context.contains_key("zoxide_query") {
                    if exit_code == Some(0) {
                        let output = String::from_utf8_lossy(&stdout);
                        let directories = parse_zoxide_output(&output);
                        self.update_directories(directories);
                    } else {
                        let error = String::from_utf8_lossy(&stderr);
                        self.set_error(format!("zoxide error: {}", error));
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn pipe(&mut self, message: PipeMessage) -> bool {
        if message.name == "filepicker_result" {
            if let (Some(payload), Some(request_id)) =
                (message.payload, message.args.get("request_id"))
            {
                if self.is_valid_request_id(request_id) {
                    self.remove_request_id(request_id);
                    let path = PathBuf::from(payload);
                    let folder = if path.is_file() {
                        path.parent().map(|p| p.to_path_buf()).unwrap_or(path)
                    } else {
                        path
                    };
                    self.set_new_session_folder(Some(folder));
                }
            }
            true
        } else {
            false
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        ui::render(self, rows, cols);
    }
}

fn fetch_zoxide_directories() {
    let mut context = BTreeMap::new();
    context.insert("zoxide_query".to_string(), "true".to_string());
    run_command(&["zoxide", "query", "-l", "-s"], context);
}
