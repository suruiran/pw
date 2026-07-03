use ratatui::layout::Rect;
use tui_scrollview::ScrollView;

use crate::{entry_theme::EntryThemeRef, schema::Argument};

impl Argument {
    pub(crate) fn render_c_input(
        &self,
        root: &mut ScrollView,
        pid: &str,
        idx: usize,
        area: Rect,
        val: Option<&str>,
        theme: EntryThemeRef,
    ) {
    }
}
