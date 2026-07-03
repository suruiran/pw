use ratatui::{
    layout::{Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_scrollview::ScrollView;

use crate::{
    entry_theme::EntryThemeRef,
    model_state::{ModelState, get_argvs},
    schema::Argument,
};

impl Argument {
    pub(crate) fn height(&self, model_state: &ModelState, path: &[String]) -> u16 {
        let height: u16 = 4;
        let mut vsize: u16 = 1;
        if self.repeatable.unwrap_or(false) {
            if let Some(argv) = get_argvs(model_state, path, &self.name) {
                vsize = argv.len() as u16;
            }
        }
        if vsize < 1 {
            vsize = 1;
        }
        return height + (vsize - 1) * 3;
    }

    fn build_label(&self, theme: EntryThemeRef) -> Line<'static> {
        let mut line = Line::default();
        line.push_span(Span::from(format!(" {}", &self.name)).style(theme.argu_label_name_style()));
        line.push_span(
            Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
        );
        line.push_span(Span::from(self.kind.kind()).style(theme.argu_label_type_style()));
        if self.required.unwrap_or(false) {
            line.push_span(
                Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
            );
            line.push_span(
                Span::from(theme.argu_label_required_token())
                    .style(theme.argu_label_required_style()),
            );
        }

        if self.repeatable.unwrap_or(false) {
            line.push_span(
                Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
            );
            line.push_span(
                Span::from(theme.argu_label_add_value_token())
                    .style(theme.argu_label_add_value_style()),
            );
        }

        line.push_span(
            Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
        );
        line.push_span(
            Span::from(theme.argu_label_desc_indicator_token())
                .style(theme.argu_label_desc_indicator_style()),
        );
        line.push_span(Span::from(" "));
        return line;
    }

    fn render_input(
        &self,
        root: &mut ScrollView,
        idx: usize,
        area: Rect,
        val: Option<&str>,
        theme: EntryThemeRef,
        path: &[String],
    ) {
        let input_id = Argument::mk_input_id(self, path, idx);
        root.render_widget(
            Paragraph::new(val.as_ref().map_or("".to_string(), |v| v.to_string())).block(
                Block::new()
                    .borders(Borders::BOTTOM)
                    .border_style(theme.clone().argu_input_border_style(false)),
            ),
            area,
        );
    }

    pub(crate) fn render(
        &self,
        root: &mut ScrollView,
        area: Rect,
        model_state: &ModelState,
        path: &[String],
        theme: EntryThemeRef,
    ) {
        root.render_widget(
            Block::bordered()
                .title(self.build_label(theme.clone()))
                .border_style(theme.clone().argu_wrapper_border_style()),
            area,
        );

        let mut constraints = vec![];

        let argv = get_argvs(model_state, path, &self.name);
        let value_count = argv.as_ref().map_or_else(|| 0, |v| v.len());
        let mut input_count = 1;
        if self.repeatable.unwrap_or(false) {
            input_count = value_count;
        }
        if input_count < 1 {
            input_count = 1;
        }

        for _ in 0..input_count {
            constraints.push(ratatui::layout::Constraint::Length(3));
        }

        let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints).split(
            area.inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );
        let mut lidx: usize = 0;

        for vidx in 0..input_count {
            self.render_input(
                root,
                lidx,
                layouts[lidx],
                argv.as_ref().map_or_else(
                    || None,
                    move |vs| {
                        if vidx >= vs.len() {
                            return None;
                        }
                        return Some(vs[vidx].as_str());
                    },
                ),
                theme.clone(),
                path,
            );
            lidx += 1;
        }
    }
}

// static methods
impl Argument {
    pub(crate) fn mk_input_id(argv: &Argument, path: &[String], idx: usize) -> String {
        return format!("{}:{}#{}", path.join("/"), &argv.name, idx);
    }
}
