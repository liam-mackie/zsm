use zellij_tile::prelude::{Palette, Text};

/// Color theme for the plugin UI
/// Uses indexed colors that map to user's Zellij theme:
/// 0 = dim/subtle, 1 = warning/orange, 2 = success/green, 3 = info/blue
#[derive(Copy, Clone, Debug)]
pub struct Theme;

impl Theme {
    /// Create theme from a palette (not used, colors are indexed)
    pub fn new(_palette: Palette) -> Self {
        Self
    }

    /// Text for warnings (color index 1 = orange/yellow)
    pub fn warning(&self, text: &str) -> Text {
        Text::new(text).color_range(1, ..)
    }

    /// Text for titles (color index 3 = blue/cyan)
    pub fn title(&self, text: &str) -> Text {
        Text::new(text).color_range(3, ..)
    }

    /// Text for regular content (default color)
    pub fn content(&self, text: &str) -> Text {
        Text::new(text)
    }

    /// Text for current session (color index 2 = green - active)
    pub fn current_session(&self, text: &str) -> Text {
        Text::new(text).color_range(2, ..)
    }

    /// Text for available session (color index 3 = cyan - available)
    pub fn available_session(&self, text: &str) -> Text {
        Text::new(text).color_range(3, ..)
    }

    /// Text for search highlights (color index 3 = yellow/blue)
    pub fn highlight(&self, text: Text, indices: Vec<usize>) -> Text {
        text.color_indices(3, indices)
    }
}