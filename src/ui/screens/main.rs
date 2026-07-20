use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::app::AppState;
use crate::domain::{DisplayItem, TargetMode};
use crate::ui::widgets::{render_deletion_dialog, render_list, render_search_bar, ListItem};
use crate::ui::Theme;

pub fn render_main_screen(state: &AppState, rows: usize, cols: usize, theme: &Theme) {
    if let Some(pending) = state.sessions().pending_deletion() {
        render_deletion_dialog(pending, rows, cols, theme);
        return;
    }

    if let Some(error) = state.error() {
        print_text_with_coordinates(theme.warning(error), 0, 0, None, None);
        return;
    }

    let title = if state.config().dev_mode {
        let mode_label = match state.config().mode {
            TargetMode::Session => "[Session]",
            TargetMode::Tab => "[Tab]",
        };
        theme.title(&format!("Zoxide Session Manager {}", mode_label))
    } else {
        theme.title("Zoxide Session Manager")
    };
    print_text_with_coordinates(title, 0, 0, None, None);

    let search_row = 2;
    render_search_bar(state.search_term(), search_row, theme);

    let list_start = 4;
    let list_max_rows = rows.saturating_sub(list_start + 2);

    let items = state.display_items();
    let selected = state.selected_index();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let indices = state.match_indices(i);
            ListItem {
                text: format_item(item, theme, indices),
                is_selected: Some(i) == selected,
            }
        })
        .collect();

    render_list(&list_items, list_start, list_max_rows, selected, theme);

    let help_row = rows.saturating_sub(1);
    let help_text = if state.config().dev_mode {
        "↑↓ Navigate | Enter Select | Ctrl+Enter Quick | Del Delete | Ctrl+M Mode | Esc Close"
    } else {
        "↑↓ Navigate | Enter Select | Ctrl+Enter Quick | Del Delete | Esc Close"
    };
    let help = theme.dim(help_text);
    print_text_with_coordinates(help, 0, help_row, None, None);

    if state.config().dev_mode {
        let version_str = build_version();
        let x = cols.saturating_sub(version_str.len());
        let version = Text::new(&version_str);
        print_text_with_coordinates(version, x, help_row, None, None);
    }
}

fn build_version() -> String {
    if cfg!(debug_assertions) {
        format!("v{}-dev.{}", env!("CARGO_PKG_VERSION"), env!("BUILD_TIMESTAMP"))
    } else {
        format!("v{}", env!("CARGO_PKG_VERSION"))
    }
}

fn format_item(item: &DisplayItem, theme: &Theme, match_indices: Option<&[usize]>) -> Text {
    let display = item.display_text();

    if let Some(indices) = match_indices {
        // Only color matched characters so they stand out against default text.
        let len = display.chars().count();
        let clamped: Vec<usize> = indices.iter().copied().filter(|&i| i < len).collect();
        return Text::new(&display).color_indices(3, clamped);
    }

    match item {
        DisplayItem::ExistingSession {
            is_current: true, ..
        } => theme.current_session(&display),
        DisplayItem::ExistingSession { .. } => theme.session(&display),
        DisplayItem::ResurrectableSession { .. } => theme.dim(&display),
        DisplayItem::Directory { .. } => theme.content(&display),
    }
}
