use zellij_tile::prelude::{
    print_table_with_coordinates, print_text_with_coordinates, Palette, Table, Text,
};

use crate::session::SessionItem;
use crate::state::{ActiveScreen, PluginState};
use crate::ui::{Colors, Theme};

/// Main renderer for the plugin UI
pub struct PluginRenderer;

impl PluginRenderer {
    /// Render the main plugin interface
    pub fn render(state: &mut PluginState, rows: usize, cols: usize) {
        let (x, y, width, height) = Self::calculate_main_size(rows, cols);

        match state.active_screen() {
            ActiveScreen::Main => {
                Self::render_main_screen(state, x, y, width, height);
            }
            ActiveScreen::NewSession => {
                Self::render_new_session_screen(&*state, x, y, width, height);
            }
            ActiveScreen::Rename => {
                Self::render_rename_screen(&*state, x, y, width, height);
            }
        }

        // Render overlays
        if let Some(error) = state.error() {
            Self::render_error(error, x, y, width, height);
        } else if let Some(session_name) = state.session_manager().pending_deletion() {
            Self::render_deletion_confirmation(session_name, x, y, width, height, state.colors());
        }
    }

    /// Render the main screen with directory/session list
    fn render_main_screen(
        state: &mut PluginState,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        let theme = state.colors().map(Theme::new);

        // Render search indication
        let search_term = state.search_engine().search_term();
        let search_text = format!("Search: {}_", search_term);
        let search_indication = if let Some(theme) = &theme {
            theme.content(&search_text).color_range(2, ..7)
        } else {
            Text::new(&search_text).color_range(1, ..7)
        };
        print_text_with_coordinates(search_indication, x, y, None, None);

        // Render main content
        // Reserve 5 rows: 1 search bar + 1 empty + table + 2 help rows
        let table_rows = height.saturating_sub(5);
        let table = if state.search_engine().is_searching() {
            Self::render_search_results(&*state, table_rows, width, &theme)
        } else {
            Self::render_all_items(state, table_rows, width, &theme)
        };

        if state.display_items().is_empty() && !state.search_engine().is_searching() {
            let no_dirs_text = if let Some(theme) = &theme {
                theme.warning("No zoxide directories found. Make sure zoxide is installed and you have visited some directories.")
            } else {
                Text::new("No zoxide directories found. Make sure zoxide is installed and you have visited some directories.")
                    .color_range(1, ..)
            };
            print_text_with_coordinates(no_dirs_text, x, y + 2, None, None);
        } else {
            print_table_with_coordinates(table, x, y + 2, Some(width), Some(table_rows));
        }

        // Render help text (2 rows starting at y + height - 2)
        Self::render_help_text(state, x, y + height.saturating_sub(2), &theme);
    }

    /// Render new session creation screen
    fn render_new_session_screen(
        state: &PluginState,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        let colors = state
            .colors()
            .map(Colors::new)
            .unwrap_or_else(|| Colors::new(Palette::default()));
        crate::ui::components::render_new_session_block(
            state.new_session_info(),
            colors,
            height.saturating_sub(2),
            width,
            x,
            y,
        );
    }

    /// Render session rename screen
    fn render_rename_screen(
        state: &PluginState,
        x: usize,
        y: usize,
        _width: usize,
        _height: usize,
    ) {
        let theme = state.colors().map(Theme::new);

        // Render prompt
        let prompt = "Rename session:";
        let prompt_text = if let Some(theme) = &theme {
            theme.content(prompt).color_range(2, ..)
        } else {
            Text::new(prompt).color_range(2, ..)
        };
        print_text_with_coordinates(prompt_text, x, y, None, None);

        // Render current input with cursor
        let input_display = format!("{}_", state.rename_buffer());
        let input_text = if let Some(theme) = &theme {
            theme.content(&input_display)
        } else {
            Text::new(&input_display)
        };
        print_text_with_coordinates(input_text, x, y + 2, None, None);

        // Render help text
        let help = "Enter: Confirm • Esc: Cancel";
        let help_text = if let Some(theme) = &theme {
            theme.content(help).color_range(1, ..)
        } else {
            Text::new(help).color_range(1, ..)
        };
        print_text_with_coordinates(help_text, x, y + 4, None, None);
    }

    /// Render search results table
    fn render_search_results(
        state: &PluginState,
        table_rows: usize,
        table_width: usize,
        theme: &Option<Theme>,
    ) -> Table {
        let mut table = Table::new().add_row(vec!["Directory/Session"]);
        let results = state.search_engine().results();
        let selected_index = state.search_engine().selected_index();

        // Calculate name column width from search result items
        let items: Vec<_> = results.iter().map(|r| r.item.clone()).collect();
        let name_col_width = Self::calculate_name_column_width(&items);

        let (first_row, last_row) =
            Self::calculate_render_range(table_rows, results.len(), selected_index);

        for i in first_row..last_row {
            if let Some(result) = results.get(i) {
                let is_selected = Some(i) == selected_index;
                let mut table_cells = vec![Self::render_search_result_item(
                    &result.item,
                    &result.indices,
                    table_width.saturating_sub(4),
                    name_col_width,
                    theme,
                )];

                if is_selected {
                    table_cells = table_cells.drain(..).map(|t| t.selected()).collect();
                }

                table = table.add_styled_row(table_cells);
            }
        }

        table
    }

    /// Render all items table
    fn render_all_items(
        state: &mut PluginState,
        table_rows: usize,
        table_width: usize,
        theme: &Option<Theme>,
    ) -> Table {
        let mut table = Table::new().add_row(vec!["Directory/Session"]);
        let items = state.display_items();
        let selected_index = state.selected_index();

        // Calculate column width once for all items
        let name_col_width = Self::calculate_name_column_width(&items);

        let (first_row, last_row) =
            Self::calculate_render_range(table_rows, items.len(), selected_index);

        for i in first_row..last_row {
            if let Some(item) = items.get(i) {
                let is_selected = Some(i) == selected_index;
                let mut table_cells = vec![Self::render_item(
                    item,
                    table_width.saturating_sub(4),
                    name_col_width,
                    theme,
                )];

                if is_selected {
                    table_cells = table_cells.drain(..).map(|t| t.selected()).collect();
                }

                table = table.add_styled_row(table_cells);
            }
        }

        table
    }

    /// Calculate the width of the name column based on the longest session name
    fn calculate_name_column_width(items: &[SessionItem]) -> usize {
        let max_name_len = items
            .iter()
            .filter_map(|item| match item {
                SessionItem::ExistingSession { name, .. } => Some(name.len() + 2), // "● " prefix
                SessionItem::ResurrectableSession { name, .. } => Some(name.len() + 2), // "↺ " prefix
                SessionItem::Directory { .. } => None, // Directories don't use columnar format
            })
            .max()
            .unwrap_or(0);

        // No cap - session names are never truncated
        max_name_len
    }

    /// Render a search result item
    ///
    /// Search indices are based on the OLD format (e.g., "● name (directory)") but we now
    /// render in columnar format (e.g., "● name    directory"). This function adjusts
    /// indices to account for the format change.
    fn render_search_result_item(
        item: &SessionItem,
        indices: &[usize],
        max_width: usize,
        name_col_width: usize,
        theme: &Option<Theme>,
    ) -> Text {
        let mut text = Self::render_item(item, max_width, name_col_width, theme);

        // Apply search highlighting
        if !indices.is_empty() {
            // Indices are based on old format - need to adjust for new columnar format
            let adjusted_indices = match item {
                SessionItem::ExistingSession {
                    name, directory, ..
                } => {
                    // Old format: "● name (directory)" or "○ name (directory)"
                    // New format: "● name    directory" (padded to name_col_width + 2 gap)
                    //
                    // Index mapping:
                    // - 0-1: bullet prefix (unchanged)
                    // - 2 to 2+name.len(): session name (unchanged, but capped by column width)
                    // - Old: name.len()+2 to name.len()+3 = " ("
                    // - New: name.len()+2 to name_col_width+2 = padding spaces
                    // - Old: dir starts at 2+name.len()+2 = 4+name.len()
                    // - New: dir starts at name_col_width+2 (after padding)

                    let prefix_len = 2; // "● " or "○ "
                    let old_dir_start = prefix_len + name.len() + 2; // " ("
                    let new_dir_start = name_col_width + 2; // after padding gap

                    indices
                        .iter()
                        .filter_map(|&idx| {
                            if idx < prefix_len {
                                // Bullet prefix - unchanged
                                Some(idx)
                            } else if idx < prefix_len + name.len() {
                                // Session name - check if truncated
                                let name_display_len =
                                    (prefix_len + name.len()).min(name_col_width);
                                if idx < name_display_len {
                                    Some(idx)
                                } else {
                                    None // Truncated
                                }
                            } else if idx >= old_dir_start && idx < old_dir_start + directory.len()
                            {
                                // Directory part - remap to new position
                                let dir_idx = idx - old_dir_start;
                                let dir_col_max = max_width.saturating_sub(new_dir_start);

                                // Handle directory truncation
                                if directory.len() > dir_col_max && dir_col_max > 10 {
                                    // Directory is truncated with "..." prefix
                                    let truncated_start =
                                        directory.len().saturating_sub(dir_col_max - 3);
                                    if dir_idx >= truncated_start {
                                        Some(new_dir_start + 3 + (dir_idx - truncated_start))
                                    } else {
                                        None // Index in truncated part
                                    }
                                } else if dir_idx < directory.len() {
                                    Some(new_dir_start + dir_idx)
                                } else {
                                    None
                                }
                            } else {
                                // " " or ")" in old format - skip
                                None
                            }
                        })
                        .collect()
                }
                SessionItem::ResurrectableSession { name, duration } => {
                    // Old format: "↺ name (created X ago)"
                    // New format: "↺ name    X ago"
                    //
                    // The duration part changed format, so indices after the name
                    // may not match well. Only highlight name portion reliably.

                    let prefix_len = 2; // "↺ "
                    let old_dur_start = prefix_len + name.len() + 10; // " (created "
                    let new_dur_start = name_col_width + 2;

                    let duration_str = format!("{} ago", humantime::format_duration(*duration));

                    indices
                        .iter()
                        .filter_map(|&idx| {
                            if idx < prefix_len {
                                // Prefix - unchanged
                                Some(idx)
                            } else if idx < prefix_len + name.len() {
                                // Session name - check truncation
                                let name_display_len =
                                    (prefix_len + name.len()).min(name_col_width);
                                if idx < name_display_len {
                                    Some(idx)
                                } else {
                                    None
                                }
                            } else if idx >= old_dur_start {
                                // Duration portion - remap
                                // Old had "created X ago", new has just "X ago"
                                // The "created " is 8 chars, so subtract 8 from offset
                                let old_offset = idx - old_dur_start;
                                if old_offset < duration_str.len() {
                                    Some(new_dur_start + old_offset)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                }
                SessionItem::Directory { path, .. } => {
                    // Directories don't use columnar format - handle truncation only
                    if path.len() > max_width && max_width > 10 {
                        // Path is truncated with "..."
                        let truncated_start = path.len().saturating_sub(max_width - 3);
                        indices
                            .iter()
                            .filter_map(|&idx| {
                                if idx >= truncated_start {
                                    Some(idx - truncated_start + 3) // +3 for "..."
                                } else {
                                    None // Index is in truncated part
                                }
                            })
                            .collect()
                    } else {
                        indices.to_vec()
                    }
                }
            };

            if !adjusted_indices.is_empty() {
                if let Some(theme) = theme {
                    text = theme.highlight(text, adjusted_indices);
                } else {
                    text = text.color_indices(3, adjusted_indices);
                }
            }
        }

        text
    }

    /// Render a session item with columnar alignment
    fn render_item(
        item: &SessionItem,
        max_width: usize,
        name_col_width: usize,
        theme: &Option<Theme>,
    ) -> Text {
        match item {
            SessionItem::ExistingSession {
                name,
                directory,
                is_current,
            } => {
                let prefix = if *is_current { "● " } else { "○ " };
                let name_display = format!("{}{}", prefix, name);

                // Calculate remaining width for directory (after name column + 2 char gap)
                let dir_col_start = name_col_width + 2;
                let dir_max_width = max_width.saturating_sub(dir_col_start);

                // Truncate directory from left if needed (names are never truncated)
                let dir_display = if directory.len() > dir_max_width && dir_max_width > 10 {
                    format!(
                        "...{}",
                        &directory[directory.len().saturating_sub(dir_max_width - 3)..]
                    )
                } else {
                    directory.clone()
                };

                // Format with padding: name padded to column width, then directory
                let display_text = format!(
                    "{:<width$}  {}",
                    name_display,
                    dir_display,
                    width = name_col_width
                );

                // Color session name only (after bullet)
                // Emphasis colors: 0=orange, 1=cyan, 2=green, 3=pink (theme-dependent)
                let color_idx = if *is_current { 2 } else { 1 };
                let name_end = 2 + name.len();
                Text::new(&display_text).color_range(color_idx, 2..name_end)
            }
            SessionItem::ResurrectableSession { name, duration } => {
                let prefix = "↺ ";
                let name_display = format!("{}{}", prefix, name);

                // Format duration info for second column
                let duration_str = format!("{} ago", humantime::format_duration(*duration));

                // Format with padding (names are never truncated)
                let display_text = format!(
                    "{:<width$}  {}",
                    name_display,
                    duration_str,
                    width = name_col_width
                );

                // Color just the session name portion in pink (color index 3)
                let name_end = 2 + name.len();
                if let Some(theme) = theme {
                    theme.content(&display_text).color_range(3, 2..name_end)
                } else {
                    Text::new(&display_text).color_range(3, 2..name_end)
                }
            }
            SessionItem::Directory { path, .. } => {
                // Directories don't use columnar format - just display the path
                let display_path = if path.len() > max_width && max_width > 10 {
                    format!("...{}", &path[path.len().saturating_sub(max_width - 3)..])
                } else {
                    path.to_string()
                };

                if let Some(theme) = theme {
                    theme.content(&display_path)
                } else {
                    Text::new(&display_path)
                }
            }
        }
    }

    /// Apply color to keys in help text using character indices (not byte indices)
    fn color_keys(text: &str, keys: &[&str], color_idx: usize) -> Text {
        let mut result = Text::new(text);
        for key in keys {
            if let Some(byte_start) = text.find(key) {
                // Convert byte position to character position
                let char_start = text[..byte_start].chars().count();
                let char_len = key.chars().count();
                result = result.color_range(color_idx, char_start..char_start + char_len);
            }
        }
        result
    }

    /// Render help text on two rows (row 1: navigation, row 2: actions)
    fn render_help_text(state: &PluginState, x: usize, y: usize, _theme: &Option<Theme>) {
        let (row1, row2, keys1, keys2) = if state.display_items().is_empty() {
            (
                "<Enter> Create, <Esc> Exit",
                "<Ctrl+Enter> Quick create",
                vec!["<Enter>", "<Esc>"],
                vec!["<Ctrl+Enter>"],
            )
        } else {
            (
                "<↑↓> Navigate, <Enter> Switch/New, <Esc> Exit",
                "<Ctrl+Enter> Quick, <Alt+r> Rename, <Alt+d> Dead, <Ctrl+r> Reload, <Del> Kill",
                vec!["<↑↓>", "<Enter>", "<Esc>"],
                vec!["<Ctrl+Enter>", "<Alt+r>", "<Alt+d>", "<Ctrl+r>", "<Del>"],
            )
        };

        let text1 = Self::color_keys(row1, &keys1, 0);
        let text2 = Self::color_keys(row2, &keys2, 0);
        print_text_with_coordinates(text1, x, y, None, None);
        print_text_with_coordinates(text2, x, y + 1, None, None);
    }

    /// Render error message
    fn render_error(error: &str, x: usize, y: usize, _width: usize, height: usize) {
        let dialog_y = y + height / 2;
        let error_text = Text::new(error).color_range(1, ..);
        print_text_with_coordinates(error_text, x, dialog_y, None, None);
    }

    /// Render deletion confirmation dialog
    fn render_deletion_confirmation(
        session_name: &str,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        _colors: Option<Palette>,
    ) {
        let dialog_width = std::cmp::min(60, width.saturating_sub(4));
        let content_width = dialog_width.saturating_sub(4); // Space for borders + padding

        let message = format!("Kill session '{}'?", session_name);
        let warning =
            "If this is a resurrectable session, it will be deleted. This action cannot be undone.";
        let prompt = "Press 'y' to confirm, 'n' or Esc to cancel";

        // Wrap warning text to fit dialog
        let wrapped_warning = Self::wrap_text(warning, content_width);

        // Dynamic height: top border + message + warning lines + blank + prompt + bottom border
        let dialog_height = 4 + wrapped_warning.len();
        let dialog_x = x + (width.saturating_sub(dialog_width)) / 2;
        let dialog_y = y + (height.saturating_sub(dialog_height)) / 2;

        let inner_width = dialog_width.saturating_sub(2);

        // Build dialog lines
        let mut dialog_lines = vec![
            "┌".to_string() + &"─".repeat(inner_width) + "┐",
            format!("│{:^width$}│", message, width = inner_width),
        ];

        // Add wrapped warning lines
        for line in &wrapped_warning {
            dialog_lines.push(format!("│{:^width$}│", line, width = inner_width));
        }

        // Add blank line and prompt
        dialog_lines.push(format!("│{:^width$}│", "", width = inner_width));
        dialog_lines.push(format!("│{:^width$}│", prompt, width = inner_width));
        dialog_lines.push("└".to_string() + &"─".repeat(inner_width) + "┘");

        for (i, line) in dialog_lines.iter().enumerate() {
            let text = Text::new(line).color_range(1, ..);
            print_text_with_coordinates(text, dialog_x, dialog_y + i, None, None);
        }
    }

    /// Wrap text to fit within max_width, breaking at word boundaries
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        if text.len() <= max_width {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Calculate main UI size
    fn calculate_main_size(rows: usize, cols: usize) -> (usize, usize, usize, usize) {
        let width = cols;
        let x = 0;
        let y = 0;
        let height = rows.saturating_sub(y);
        (x, y, width, height)
    }

    /// Calculate which rows to render for pagination
    fn calculate_render_range(
        table_rows: usize,
        items_len: usize,
        selected_index: Option<usize>,
    ) -> (usize, usize) {
        if table_rows <= items_len {
            let row_count_to_render = table_rows.saturating_sub(1); // 1 for the title
            let first_row_index = selected_index
                .unwrap_or(0)
                .saturating_sub(row_count_to_render / 2);
            let last_row_index = first_row_index + row_count_to_render;
            (first_row_index, last_row_index)
        } else {
            (0, items_len)
        }
    }
}
