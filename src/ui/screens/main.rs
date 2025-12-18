use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::app::AppState;
use crate::domain::DisplayItem;
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

    let title = theme.title("Zoxide Session Manager");
    print_text_with_coordinates(title, 0, 0, None, None);

    let search_row = 2;
    render_search_bar(state.search().term(), search_row, theme);

    let list_start = 4;
    let list_max_rows = rows.saturating_sub(list_start + 2);

    let items = state.display_items();
    let selected = state.selected_index();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| ListItem {
            text: format_item(item, theme),
            is_selected: Some(i) == selected,
        })
        .collect();

    render_list(&list_items, list_start, list_max_rows, selected, theme);

    let help_row = rows.saturating_sub(1);
    let help = theme.dim("↑↓ Navigate | Enter Select | Ctrl+Enter Quick | Del Delete | Esc Close");
    print_text_with_coordinates(help, 0, help_row, None, None);
}

fn format_item(item: &DisplayItem, theme: &Theme) -> Text {
    match item {
        DisplayItem::ExistingSession {
            name, is_current, ..
        } => {
            let prefix = if *is_current { "● " } else { "○ " };
            let text = format!("{}{}", prefix, name);
            if *is_current {
                theme.current_session(&text)
            } else {
                theme.session(&text)
            }
        }
        DisplayItem::ResurrectableSession { name, duration } => {
            let text = format!("↺ {} ({})", name, humantime::format_duration(*duration));
            theme.dim(&text)
        }
        DisplayItem::Directory { path, .. } => theme.content(path),
    }
}
