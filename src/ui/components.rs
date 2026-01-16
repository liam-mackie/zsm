use crate::new_session_info::NewSessionInfo;
use zellij_tile::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Colors;

impl Colors {
    pub fn new(_palette: Palette) -> Self {
        Colors
    }

    pub fn shortcuts(&self, text: &str) -> Text {
        Text::new(text).color_range(3, ..)
    }
}

pub fn render_new_session_block(
    new_session_info: &NewSessionInfo,
    colors: Colors,
    max_rows_of_new_session_block: usize,
    max_cols_of_new_session_block: usize,
    x: usize,
    y: usize,
) {
    let _enter = colors.shortcuts("<ENTER>");
    if new_session_info.entering_new_session_name() {
        let prompt = "New session name:";
        let long_instruction = "when done, blank for random";
        let new_session_name = new_session_info.name();
        if max_cols_of_new_session_block > 70 {
            let session_name_text = Text::new(format!(
                "{} {}_ (<ENTER> {})",
                prompt, new_session_name, long_instruction
            ))
            .color_range(3, ..prompt.len())
            .color_range(
                0,
                prompt.len() + 1..prompt.len() + 1 + new_session_name.len(),
            )
            .color_range(
                3,
                prompt.len() + new_session_name.len() + 4
                    ..prompt.len() + new_session_name.len() + 11,
            );
            print_text_with_coordinates(session_name_text, x, y + 1, None, None);
        } else {
            let session_name_text = Text::new(format!("{} {}_ <ENTER>", prompt, new_session_name))
                .color_range(3, ..prompt.len())
                .color_range(
                    0,
                    prompt.len() + 1..prompt.len() + 1 + new_session_name.len(),
                )
                .color_range(3, prompt.len() + new_session_name.len() + 3..);
            print_text_with_coordinates(session_name_text, x, y + 1, None, None);
        }
    } else if new_session_info.entering_layout_search_term() {
        let new_session_name = if new_session_info.name().is_empty() {
            "<RANDOM>"
        } else {
            new_session_info.name()
        };
        let prompt = "New session name:";
        let session_name_text = Text::new(format!(
            "{} {} (Ctrl+<R> to correct)",
            prompt, new_session_name
        ))
        .color_range(2, ..prompt.len())
        .color_range(
            1,
            prompt.len() + 1..prompt.len() + 1 + new_session_name.len(),
        )
        .color_range(
            3,
            prompt.len() + new_session_name.len() + 3..prompt.len() + new_session_name.len() + 12,
        );
        print_text_with_coordinates(session_name_text, x, y + 1, None, None);

        render_layout_selection_list(
            new_session_info,
            max_rows_of_new_session_block.saturating_sub(8),
            max_cols_of_new_session_block,
            x,
            y + 1,
        );
    }
    render_new_session_folder_prompt(
        new_session_info,
        colors,
        x,
        (y + max_rows_of_new_session_block).saturating_sub(3),
        max_cols_of_new_session_block,
    );
}

pub fn render_layout_selection_list(
    new_session_info: &NewSessionInfo,
    max_rows_of_new_session_block: usize,
    max_cols_of_new_session_block: usize,
    x: usize,
    y: usize,
) {
    let layout_search_term = new_session_info.layout_search_term();
    let layout_indication_line = if max_cols_of_new_session_block > 73 {
        Text::new(format!(
            "New session layout: {}_ (Search and select from list, <ENTER> when done)",
            layout_search_term
        ))
        .color_range(2, ..20)
        .color_range(1, 20..20 + layout_search_term.len())
        .color_range(
            3,
            52 + layout_search_term.len()..59 + layout_search_term.len(),
        )
    } else {
        Text::new(format!(
            "New session layout: {}_ <ENTER>",
            layout_search_term
        ))
        .color_range(2, ..20)
        .color_range(1, 20..20 + layout_search_term.len())
        .color_range(3, 22 + layout_search_term.len()..)
    };
    print_text_with_coordinates(layout_indication_line, x, y + 1, None, None);

    let mut table = Table::new();
    for (i, (layout_info, indices, is_selected)) in new_session_info
        .layouts_to_render(max_rows_of_new_session_block)
        .into_iter()
        .enumerate()
    {
        let layout_name = layout_info.name();
        let is_builtin = layout_info.is_builtin();
        if i > max_rows_of_new_session_block.saturating_sub(1) {
            break;
        } else {
            let mut layout_cell = if is_builtin {
                Text::new(format!("{} (built-in)", layout_name))
                    .color_range(1, 0..layout_name.len())
                    .color_range(0, layout_name.len() + 1..)
                    .color_indices(3, indices)
            } else {
                Text::new(layout_name.to_string())
                    .color_range(1, ..)
                    .color_indices(3, indices)
            };
            if is_selected {
                layout_cell = layout_cell.selected();
            }
            table = table.add_styled_row(vec![layout_cell]);
        }
    }
    print_table_with_coordinates(
        table,
        x,
        y + 3,
        Some(max_cols_of_new_session_block),
        Some(max_rows_of_new_session_block),
    );
}

pub fn render_new_session_folder_prompt(
    new_session_info: &NewSessionInfo,
    _colors: Colors,
    x: usize,
    y: usize,
    max_cols: usize,
) {
    match &new_session_info.new_session_folder {
        Some(folder) => {
            let short_folder_prompt = "New session folder:";
            let folder_path = folder.to_string_lossy();
            if max_cols > short_folder_prompt.len() + folder_path.len() + 40 {
                let folder_text = Text::new(format!(
                    "{} {} (Ctrl+<f> to change, Ctrl+<c> to clear)",
                    short_folder_prompt, folder_path
                ))
                .color_range(2, ..short_folder_prompt.len())
                .color_range(
                    1,
                    short_folder_prompt.len() + 1
                        ..short_folder_prompt.len() + 1 + folder_path.len(),
                )
                .color_range(
                    3,
                    short_folder_prompt.len() + folder_path.len() + 3
                        ..short_folder_prompt.len() + folder_path.len() + 11,
                )
                .color_range(
                    3,
                    short_folder_prompt.len() + folder_path.len() + 23
                        ..short_folder_prompt.len() + folder_path.len() + 31,
                );
                print_text_with_coordinates(folder_text, x, y + 1, None, None);
            } else {
                let folder_text =
                    Text::new(format!("{} {} Ctrl+<f>", short_folder_prompt, folder_path))
                        .color_range(2, ..short_folder_prompt.len())
                        .color_range(
                            1,
                            short_folder_prompt.len() + 1
                                ..short_folder_prompt.len() + 1 + folder_path.len(),
                        )
                        .color_range(3, short_folder_prompt.len() + folder_path.len() + 2..);
                print_text_with_coordinates(folder_text, x, y + 1, None, None);
            }
        }
        None => {
            let folder_prompt = "New session folder (optional):";
            let folder_text = Text::new(format!("{} Ctrl+<f> to select", folder_prompt))
                .color_range(2, ..folder_prompt.len())
                .color_range(3, folder_prompt.len() + 1..folder_prompt.len() + 9);
            print_text_with_coordinates(folder_text, x, y + 1, None, None);
        }
    }
}
