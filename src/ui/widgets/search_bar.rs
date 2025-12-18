use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::ui::Theme;

pub fn render_search_bar(term: &str, row: usize, theme: &Theme) {
    let display = if term.is_empty() {
        theme.dim("Type to search...")
    } else {
        Text::new(&format!("Search: {}▌", term))
    };
    print_text_with_coordinates(display, 0, row, None, None);
}
