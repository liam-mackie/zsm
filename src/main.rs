mod config;
mod new_session_info;
mod session;
mod state;
mod ui;
mod zoxide;

use state::PluginState;
use std::collections::BTreeMap;
use ui::PluginRenderer;
use zellij_tile::prelude::*;

register_plugin!(PluginState);

impl ZellijPlugin for PluginState {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.initialize(configuration);

        // Request permissions - same as session-manager
        request_permission(&[
            PermissionType::RunCommands,                  // run zoxide command
            PermissionType::ReadApplicationState,         // read current sessions/layouts
            PermissionType::ChangeApplicationState,       // create and switch sessions
            PermissionType::MessageAndLaunchOtherPlugins, // launch filepicker plugin
        ]);

        subscribe(&[
            EventType::ModeUpdate,
            EventType::SessionUpdate,
            EventType::Key,
            EventType::RunCommandResult,
            EventType::PermissionRequestResult,
        ]);

        // Don't fetch zoxide directories immediately - wait for permissions
    }

    // Event handlers
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::ModeUpdate(mode_info) => {
                self.set_colors(mode_info.style.colors.into());
                should_render = true;
            }
            Event::Key(key) => {
                should_render = self.handle_key(key);
            }
            Event::PermissionRequestResult(permission_status) => {
                match permission_status {
                    PermissionStatus::Granted => {
                        // Now that we have permissions, fetch zoxide directories
                        self.fetch_zoxide_directories();
                        should_render = true;
                    }
                    PermissionStatus::Denied => {
                        self.set_error(
                            "RunCommands permission denied - cannot fetch zoxide directories"
                                .to_string(),
                        );
                        should_render = true;
                    }
                }
            }
            Event::SessionUpdate(session_infos, resurrectable_session_infos) => {
                // Only render if sessions actually changed (handles Zellij's inconsistent updates)
                let sessions_changed = self.update_sessions(session_infos);
                let resurrectable_changed =
                    self.update_resurrectable_sessions(resurrectable_session_infos);
                should_render = sessions_changed || resurrectable_changed;
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                if context.contains_key("zoxide_query") {
                    if exit_code == Some(0) {
                        let stdout_str = String::from_utf8_lossy(&stdout);
                        self.process_zoxide_output(&stdout_str);
                        should_render = true;
                    } else {
                        let stderr_str = String::from_utf8_lossy(&stderr);
                        self.set_error(format!(
                            "Failed to run zoxide (is it installed?): {}",
                            stderr_str
                        ));
                        should_render = true;
                    }
                }
            }
            _ => (),
        }

        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        // Handle filepicker results for new session creation
        if pipe_message.name == "filepicker_result" {
            if let (Some(payload), Some(request_id)) =
                (pipe_message.payload, pipe_message.args.get("request_id"))
            {
                // Check if this request ID is valid for our plugin
                if self.is_valid_request_id(request_id) {
                    self.remove_request_id(request_id);
                    let selected_path = std::path::PathBuf::from(payload);

                    // Determine if we should use the path or its parent directory
                    let session_folder = if selected_path.exists() {
                        // Path exists, check if it's a file or directory
                        if selected_path.is_file() {
                            // If it's a file, use the parent directory
                            selected_path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or(selected_path)
                        } else {
                            // It's a directory, use it directly
                            selected_path
                        }
                    } else {
                        // Path doesn't exist, try to infer from extension or structure
                        if selected_path.extension().is_some() {
                            // Has an extension, likely a file - use parent directory
                            selected_path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or(selected_path)
                        } else {
                            // No extension, assume it's a directory
                            selected_path
                        }
                    };

                    self.set_new_session_folder(Some(session_folder));
                }
            }
            true
        } else {
            false
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        PluginRenderer::render(self, rows, cols);
    }
}

impl PluginState {
    fn fetch_zoxide_directories(&mut self) {
        let mut context = BTreeMap::new();
        context.insert("zoxide_query".to_string(), "true".to_string());
        run_command(&["zoxide", "query", "-l", "-s"], context);
    }

    fn process_zoxide_output(&mut self, output: &str) {
        let mut directories = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // zoxide output format: "score path"
            let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
            if parts.len() == 2 {
                if let Ok(score) = parts[0].parse::<f64>() {
                    let path = parts[1];

                    directories.push(zoxide::ZoxideDirectory {
                        ranking: score,
                        directory: path.to_string(),
                        session_name: String::new(), // Will be set by smart naming
                    });
                }
            }
        }

        // Generate smart session names before sorting
        self.generate_smart_session_names(&mut directories);

        // Sort by score in descending order (higher scores first)
        directories.sort_by(|a, b| {
            b.ranking
                .partial_cmp(&a.ranking)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.update_zoxide_directories(directories);
    }

    fn generate_smart_session_names(&self, directories: &mut [zoxide::ZoxideDirectory]) {
        use std::collections::HashMap;

        // First pass: collect all basenames and find conflicts
        let mut basename_groups: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, dir) in directories.iter().enumerate() {
            let basename = std::path::Path::new(&dir.directory)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            basename_groups.entry(basename).or_default().push(i);
        }

        // Second pass: generate names with context for conflicts and nested directories
        for (basename, indices) in basename_groups {
            if indices.len() == 1 {
                // Check if this directory is nested within another zoxide directory
                let i = indices[0];
                let path = &directories[i].directory;
                if self.is_nested_in_zoxide_directories(path, directories) {
                    // Add context for nested directories
                    let smart_name = self.generate_context_aware_name(path, directories, &indices);
                    directories[i].session_name = smart_name;
                } else {
                    // No conflict and not nested, use basename
                    directories[i].session_name = basename;
                }
            } else {
                // Resolve conflicts with minimal context
                for &i in &indices {
                    let path = &directories[i].directory;
                    let smart_name = self.generate_context_aware_name(path, directories, &indices);
                    directories[i].session_name = smart_name;
                }
            }
        }
    }

    fn is_nested_in_zoxide_directories(
        &self,
        path: &str,
        all_directories: &[zoxide::ZoxideDirectory],
    ) -> bool {
        let normalized_path = self.normalize_path(path);
        let path_obj = std::path::Path::new(&normalized_path);

        for dir in all_directories {
            if dir.directory == path {
                continue; // Skip self
            }

            let other_normalized = self.normalize_path(&dir.directory);
            let other_path_obj = std::path::Path::new(&other_normalized);

            // Check if current path is a subdirectory of another zoxide directory
            if path_obj.starts_with(other_path_obj) {
                return true;
            }
        }

        false
    }

    fn generate_context_aware_name(
        &self,
        path: &str,
        all_directories: &[zoxide::ZoxideDirectory],
        conflict_indices: &[usize],
    ) -> String {
        // Remove home directory if present
        let normalized_path = self.normalize_path(path);
        let segments: Vec<&str> = normalized_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if segments.is_empty() {
            return "root".to_string();
        }

        // Check if this is a nested directory (subdirectory of another zoxide directory)
        let is_nested = self.is_nested_in_zoxide_directories(path, all_directories);

        let mut required_segments = if is_nested {
            // For nested directories, start with more context (minimum 2-3 segments)
            std::cmp::min(3, segments.len())
        } else {
            1 // At least the basename for non-nested
        };

        let max_segments = segments.len();
        let separator = &self.config().session_separator;

        // Find minimal context needed to distinguish from conflicts
        for context_length in required_segments..=max_segments {
            let candidate_segments = &segments[segments.len().saturating_sub(context_length)..];
            let candidate_name = candidate_segments.join(separator);

            // Check if this name is unique among conflicts
            let mut is_unique = true;
            for &conflict_idx in conflict_indices {
                let conflict_path = &all_directories[conflict_idx].directory;
                if conflict_path == path {
                    continue; // Skip self
                }

                let conflict_normalized = self.normalize_path(conflict_path);
                let conflict_segments: Vec<&str> = conflict_normalized
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect();

                if conflict_segments.len() >= context_length {
                    let conflict_candidate_segments = &conflict_segments
                        [conflict_segments.len().saturating_sub(context_length)..];
                    let conflict_candidate_name = conflict_candidate_segments.join(separator);

                    if candidate_name == conflict_candidate_name {
                        is_unique = false;
                        break;
                    }
                }
            }

            if is_unique {
                required_segments = context_length;
                break;
            }
        }

        // For nested directories, ensure we have meaningful context even if no conflicts
        if is_nested && required_segments < 2 {
            required_segments = std::cmp::min(2, segments.len());
        }

        // Build the final name with required segments
        let final_segments = &segments[segments.len().saturating_sub(required_segments)..];
        let mut session_name = final_segments.join(separator);

        // Zellij session names depend on the location of the socket
        // Currently, the length limit is 108 bytes for Unix domain sockets
        // The name of the session + the socket path must fit within this limit
        // Since we run in a WASM runtime, there's no way to programmatically get the socket path
        // therefore, we enforce a name length of ~29 characters to stay safe
        if session_name.len() > 29 {
            session_name = self.apply_smart_truncation(&segments, required_segments);
        }

        session_name
    }

    fn normalize_path(&self, path: &str) -> String {
        let base_paths = &self.config().base_paths;

        // If no base paths configured, return the original path
        if base_paths.is_empty() {
            return path.to_string();
        }

        // Find the longest matching base path
        let mut longest_match: Option<&String> = None;
        let mut longest_match_len = 0;

        for base_path in base_paths {
            // Normalize base path (remove trailing slash)
            let normalized_base = base_path.trim_end_matches('/');

            // Check if path starts with this base path
            if path.starts_with(normalized_base) {
                // Make sure it's a directory boundary (not partial match)
                let is_directory_boundary = path.len() == normalized_base.len()
                    || path.chars().nth(normalized_base.len()) == Some('/');
                if is_directory_boundary && normalized_base.len() > longest_match_len {
                    longest_match = Some(base_path);
                    longest_match_len = normalized_base.len();
                }
            }
        }

        if let Some(base_path) = longest_match {
            let normalized_base = base_path.trim_end_matches('/');

            // If path exactly matches the base path, keep the full path
            if path == normalized_base {
                return path.to_string();
            }

            // Strip the base path and the following slash
            if let Some(stripped) = path.strip_prefix(normalized_base) {
                let stripped = stripped.strip_prefix('/').unwrap_or(stripped);
                if !stripped.is_empty() {
                    return stripped.to_string();
                }
            }
        }

        path.to_string()
    }

    fn apply_smart_truncation(&self, segments: &[&str], min_segments: usize) -> String {
        let separator = &self.config().session_separator;
        let max_length = 29;

        // Start with minimum required segments from the right
        let mut result_segments: Vec<String> = segments
            [segments.len().saturating_sub(min_segments)..]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut current_length = result_segments.join(separator).len();

        // If the initial segments are already too long, truncate them
        if current_length > max_length {
            // Try abbreviating all segments first
            result_segments = result_segments
                .iter()
                .map(|s| self.abbreviate_segment(s))
                .collect();
            current_length = result_segments.join(separator).len();

            // If still too long, truncate individual segments or remove from left
            while current_length > max_length && result_segments.len() > 1 {
                result_segments.remove(0);
                current_length = result_segments.join(separator).len();
            }

            // If still too long with just one segment, truncate it
            if current_length > max_length && result_segments.len() == 1 {
                result_segments[0].truncate(max_length);
                current_length = result_segments[0].len();
            }
        }

        // Try to add more segments from the left, abbreviating as needed
        let mut left_index = segments.len().saturating_sub(min_segments + 1);

        while current_length < max_length && left_index > 0 {
            let segment = segments[left_index];
            let abbreviated = self.abbreviate_segment(segment);

            let mut test_segments = vec![abbreviated.clone()];
            test_segments.extend(result_segments.clone());
            let test_length = test_segments.join(separator).len();

            if test_length <= max_length {
                result_segments.insert(0, abbreviated);
                current_length = test_length;
                left_index = left_index.saturating_sub(1);
            } else {
                break;
            }
        }

        // Final safety check - ensure we're at or under max_length
        let mut final_result = result_segments.join(separator);
        if final_result.len() > max_length {
            final_result.truncate(max_length);
        }

        final_result
    }

    fn abbreviate_segment(&self, segment: &str) -> String {
        if segment.len() <= 3 {
            return segment.to_string();
        }

        // For segments like "lobster-watcher", create "l-w"
        if segment.contains('-') || segment.contains('_') {
            let parts: Vec<&str> = segment.split(&['-', '_'][..]).collect();
            if parts.len() > 1 {
                return parts
                    .iter()
                    .map(|part| part.chars().next().unwrap_or('x').to_string())
                    .collect::<Vec<String>>()
                    .join("-");
            }
        }

        // For CamelCase or regular words, take first letter + consonants
        let chars: Vec<char> = segment.chars().collect();
        if chars.len() > 3 {
            let mut abbreviated = String::new();
            abbreviated.push(chars[0]);

            // Add important consonants or vowels
            for &c in &chars[1..] {
                if abbreviated.len() >= 3 {
                    break;
                }
                if c.is_alphabetic() {
                    abbreviated.push(c);
                }
            }

            if abbreviated.len() < 2 && chars.len() > 1 {
                abbreviated.push(chars[1]);
            }

            return abbreviated;
        }

        segment.to_string()
    }
}
