use ratatui::{layout::Rect, widgets::Paragraph};

use crate::schema::Argv;

impl Argv {
    pub(crate) fn label<F: FnMut(Paragraph, Rect)>(&self, id: &str, area: Rect, fnc: F) {
        let mut constraints = vec![
            // name
            ratatui::layout::Constraint::Max(20),
            // type
            ratatui::layout::Constraint::Max(10),
        ];

        if self.required.unwrap_or(false) {}
    }
}

pub(crate) fn mk_input_id(argv: &Argv, path: &[String]) -> String {
    return format!("{}:{}", path.join("/"), &argv.name);
}
