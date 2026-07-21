use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::ui::Theme;

pub struct ListItem {
    pub text: Text,
    pub is_selected: bool,
}

pub fn render_list(
    items: &[ListItem],
    start_row: usize,
    max_rows: usize,
    selected_index: Option<usize>,
    _theme: &Theme,
) {
    let (visible_start, visible_end) =
        calculate_visible_range(items.len(), selected_index.unwrap_or(0), max_rows);

    for (display_idx, item_idx) in (visible_start..visible_end).enumerate() {
        if let Some(item) = items.get(item_idx) {
            let row = start_row + display_idx;
            let text = if item.is_selected {
                item.text.clone().selected()
            } else {
                item.text.clone()
            };
            print_text_with_coordinates(text, 0, row, None, None);
        }
    }
}

fn calculate_visible_range(total: usize, selected: usize, max_rows: usize) -> (usize, usize) {
    if total <= max_rows {
        return (0, total);
    }

    let half_visible = max_rows / 2;

    let start = if selected < half_visible {
        0
    } else if selected >= total - half_visible {
        total - max_rows
    } else {
        selected - half_visible
    };

    let end = (start + max_rows).min(total);

    (start, end)
}
