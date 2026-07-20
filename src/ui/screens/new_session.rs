use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::app::AppState;
use crate::ui::widgets::{render_list, ListItem};
use crate::ui::Theme;

pub fn render_new_session_screen(state: &AppState, rows: usize, _cols: usize, theme: &Theme) {
    let title = theme.title("New Session");
    print_text_with_coordinates(title, 0, 0, None, None);

    let name_label = "Name: ";
    let name_value = if state.entering_name() {
        format!("{}▌", state.new_session_name())
    } else {
        state.new_session_name().to_string()
    };
    print_text_with_coordinates(Text::new(name_label), 0, 2, None, None);
    print_text_with_coordinates(theme.content(&name_value), name_label.len(), 2, None, None);

    let folder_label = "Folder: ";
    let folder_value = state
        .new_session_folder()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "(none)".to_string());
    print_text_with_coordinates(Text::new(folder_label), 0, 3, None, None);
    print_text_with_coordinates(theme.content(&folder_value), folder_label.len(), 3, None, None);

    if !state.entering_name() {
        let layout_label = "Layout: ";
        let search_value = if state.layout_search().is_empty() {
            "Type to filter layouts...".to_string()
        } else {
            format!("{}▌", state.layout_search())
        };
        print_text_with_coordinates(Text::new(layout_label), 0, 5, None, None);
        print_text_with_coordinates(theme.dim(&search_value), layout_label.len(), 5, None, None);

        let list_start = 7;
        let list_max_rows = rows.saturating_sub(list_start + 2);

        let layouts = state.filtered_layouts();
        let selected = state.layout_selection_index();

        let list_items: Vec<ListItem> = layouts
            .iter()
            .enumerate()
            .map(|(i, layout)| ListItem {
                text: theme.content(layout.name()),
                is_selected: Some(i) == selected,
            })
            .collect();

        render_list(&list_items, list_start, list_max_rows, selected, theme);
    }

    let help_row = rows.saturating_sub(1);
    let help = theme.dim("Enter Confirm | Ctrl+F Filepicker | Ctrl+C Clear folder | Esc Back");
    print_text_with_coordinates(help, 0, help_row, None, None);
}
