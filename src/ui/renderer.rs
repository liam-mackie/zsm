use crate::app::{AppState, Screen};
use crate::ui::screens::{render_main_screen, render_new_session_screen};
use crate::ui::Theme;

pub fn render(state: &AppState, rows: usize, cols: usize) {
    let theme = Theme::new(state.colors());

    match state.screen() {
        Screen::Main => render_main_screen(state, rows, cols, &theme),
        Screen::NewSession => render_new_session_screen(state, rows, cols, &theme),
    }
}
