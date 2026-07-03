use ratatui::style::Style;

use crate::{entry_theme::EntryThemeRef, ui::UIApp};

impl UIApp {
    pub(crate) fn style(&self, id: &str) -> Style {
        return style_by_id(self.theme.clone(), id);
    }
}

pub(crate) fn style_by_id(theme: EntryThemeRef, id: &str) -> Style {
    return Style::new();
}
