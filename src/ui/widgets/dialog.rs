use zellij_tile::prelude::{print_text_with_coordinates, Text};

use crate::ui::Theme;

pub fn render_deletion_dialog(session_name: &str, rows: usize, cols: usize, theme: &Theme) {
    let title = format!("Delete session '{}'?", session_name);
    let prompt = "(y)es / (n)o";

    let center_row = rows / 2;
    let title_col = cols.saturating_sub(title.len()) / 2;
    let prompt_col = cols.saturating_sub(prompt.len()) / 2;

    print_text_with_coordinates(theme.warning(&title), title_col, center_row, None, None);
    print_text_with_coordinates(Text::new(prompt), prompt_col, center_row + 1, None, None);
}
