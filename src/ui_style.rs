use std::rc::Rc;

use ratatui::style::Style;

use crate::{entry::Theme, ui::UIApp};

impl UIApp {
    pub(crate) fn style(&self, id: &str) -> Style {
        return style_by_id(self.theme.clone(), id);
    }
}

pub(crate) fn style_by_id(theme: Rc<Option<Theme>>, id: &str) -> Style {
    return Style::new();
}
