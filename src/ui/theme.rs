use zellij_tile::prelude::{Palette, Text};

#[derive(Copy, Clone, Debug, Default)]
pub struct Theme {
    _palette: Option<Palette>,
}

impl Theme {
    pub fn new(palette: Option<Palette>) -> Self {
        Self { _palette: palette }
    }

    pub fn warning(&self, text: &str) -> Text {
        Text::new(text).color_range(1, ..)
    }

    pub fn title(&self, text: &str) -> Text {
        Text::new(text).color_range(3, ..)
    }

    pub fn content(&self, text: &str) -> Text {
        Text::new(text)
    }

    pub fn current_session(&self, text: &str) -> Text {
        Text::new(text).color_range(2, ..)
    }

    pub fn session(&self, text: &str) -> Text {
        Text::new(text).color_range(3, ..)
    }

    pub fn selected(&self, text: &str) -> Text {
        Text::new(text).selected()
    }

    pub fn highlight(&self, text: Text, indices: Vec<usize>) -> Text {
        text.color_indices(3, indices)
    }

    pub fn dim(&self, text: &str) -> Text {
        Text::new(text).color_range(0, ..)
    }
}
