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
    pub fn render(state: &PluginState, rows: usize, cols: usize) {
        let (x, y, width, height) = Self::calculate_main_size(rows, cols);

        match state.active_screen() {
            ActiveScreen::Main => {
                Self::render_main_screen(state, x, y, width, height);
            }
            ActiveScreen::NewSession => {
                Self::render_new_session_screen(state, x, y, width, height);
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
    fn render_main_screen(state: &PluginState, x: usize, y: usize, width: usize, height: usize) {
        let theme = state.colors().map(Theme::new);

        // Render title
        let title = if let Some(theme) = &theme {
            theme.title("Zoxide Session Manager")
        } else {
            Text::new("Zoxide Session Manager").color_range(2, ..)
        };
        print_text_with_coordinates(title, x, y, None, None);

        // Render search indication
        let search_term = state.search_engine().search_term();
        let search_text = format!("Search: {}_", search_term);
        let search_indication = if let Some(theme) = &theme {
            theme.content(&search_text).color_range(2, ..7)
        } else {
            Text::new(&search_text).color_range(1, ..7)
        };
        print_text_with_coordinates(search_indication, x, y + 2, None, None);

        // Render main content
        let table_rows = height.saturating_sub(6);
        let table = if state.search_engine().is_searching() {
            Self::render_search_results(state, table_rows, width, &theme)
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
            print_text_with_coordinates(no_dirs_text, x, y + 4, None, None);
        } else {
            print_table_with_coordinates(table, x, y + 4, Some(width), Some(table_rows));
        }

        // Render help text
        Self::render_help_text(state, x, y + height.saturating_sub(1), &theme);
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

        let (first_row, last_row) =
            Self::calculate_render_range(table_rows, results.len(), selected_index);

        for i in first_row..last_row {
            if let Some(result) = results.get(i) {
                let is_selected = Some(i) == selected_index;
                let mut table_cells = vec![Self::render_search_result_item(
                    &result.item,
                    &result.indices,
                    table_width.saturating_sub(4),
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
        state: &PluginState,
        table_rows: usize,
        table_width: usize,
        theme: &Option<Theme>,
    ) -> Table {
        let mut table = Table::new().add_row(vec!["Directory/Session"]);
        let items = state.display_items();
        let selected_index = state.selected_index();

        let (first_row, last_row) =
            Self::calculate_render_range(table_rows, items.len(), selected_index);

        for i in first_row..last_row {
            if let Some(item) = items.get(i) {
                let is_selected = Some(i) == selected_index;
                let mut table_cells = vec![Self::render_item(
                    item,
                    table_width.saturating_sub(4),
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

    /// Render a search result item
    fn render_search_result_item(
        item: &SessionItem,
        indices: &[usize],
        max_width: usize,
        theme: &Option<Theme>,
    ) -> Text {
        let mut text = Self::render_item(item, max_width, theme);

        // Apply search highlighting
        if !indices.is_empty() {
            // Now indices should match the display text exactly since search matches against display text
            // But we need to handle truncation for directories
            let adjusted_indices = match item {
                SessionItem::ExistingSession { .. } => {
                    // Indices should match the display text exactly
                    indices.to_vec()
                }
                SessionItem::ResurrectableSession { .. } => {
                    // Indices should match the display text exactly
                    indices.to_vec()
                }
                SessionItem::Directory { path, .. } => {
                    // Handle truncation for long paths
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

    /// Render a session item
    fn render_item(item: &SessionItem, max_width: usize, theme: &Option<Theme>) -> Text {
        match item {
            SessionItem::ExistingSession {
                name,
                directory,
                is_current,
            } => {
                let prefix = if *is_current { "● " } else { "○ " };
                let display_text = format!("{}{} ({})", prefix, name, directory);

                let truncated_text = Self::get_truncated_text(&display_text, max_width);

                if let Some(theme) = theme {
                    if *is_current {
                        theme.current_session(&truncated_text)
                    } else {
                        theme.available_session(&truncated_text)
                    }
                } else {
                    let mut text = Text::new(&truncated_text);
                    if *is_current {
                        text = text.color_range(2, ..);
                    } else {
                        text = text.color_range(3, ..);
                    }
                    text
                }
            }
            SessionItem::ResurrectableSession { name, duration } => {
                let display_text = format!(
                    "↺ {} (created {} ago)",
                    name,
                    humantime::format_duration(*duration)
                );

                let truncated_text = Self::get_truncated_text(&display_text, max_width);

                if let Some(theme) = theme {
                    theme.available_session(&truncated_text)
                } else {
                    Text::new(&truncated_text).color_range(4, ..)
                }
            }
            SessionItem::Directory { path, .. } => {
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

    /// Render help text
    fn render_help_text(state: &PluginState, x: usize, y: usize, theme: &Option<Theme>) {
        let help_text = if state.display_items().is_empty() {
            "Type session name and press Enter • Ctrl+Enter: Quick create • Esc: Exit"
        } else {
            "↑/↓: Navigate • Enter: Switch/New • Ctrl+Enter: Quick create • Ctrl+r: reload directories • Delete: Kill • Type: Search • Esc: Exit"
        };

        let text = if let Some(theme) = theme {
            theme.content(help_text).color_range(1, ..)
        } else {
            Text::new(help_text).color_range(1, ..)
        };

        print_text_with_coordinates(text, x, y, None, None);
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
        let dialog_height = 6;
        let dialog_x = x + (width.saturating_sub(dialog_width)) / 2;
        let dialog_y = y + (height.saturating_sub(dialog_height)) / 2;

        let message = format!("Kill session '{}'?", session_name);
        let warning =
            "If this is a resurrectable session, it will be deleted. This action cannot be undone.";
        let prompt = "Press 'y' to confirm, 'n' or Esc to cancel";

        let dialog_lines = [
            "┌".to_string() + &"─".repeat(dialog_width.saturating_sub(2)) + "┐",
            format!(
                "│{:^width$}│",
                message,
                width = dialog_width.saturating_sub(2)
            ),
            format!(
                "│{:^width$}│",
                warning,
                width = dialog_width.saturating_sub(2)
            ),
            format!("│{:^width$}│", "", width = dialog_width.saturating_sub(2)),
            format!(
                "│{:^width$}│",
                prompt,
                width = dialog_width.saturating_sub(2)
            ),
            "└".to_string() + &"─".repeat(dialog_width.saturating_sub(2)) + "┘",
        ];

        for (i, line) in dialog_lines.iter().enumerate() {
            let text = Text::new(line).color_range(1, ..);
            print_text_with_coordinates(text, dialog_x, dialog_y + i, None, None);
        }
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

    fn get_truncated_text(text: &str, max_width: usize) -> String {
        if text.len() > max_width && max_width > 10 {
            format!(
                "{}...{}",
                &text[..10],
                &text[text.len().saturating_sub(max_width - 13)..]
            )
        } else {
            text.to_string()
        }
    }
}
