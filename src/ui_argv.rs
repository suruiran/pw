use ratatui::{
    layout::{Layout, Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_scrollview::ScrollView;
use unicode_width::UnicodeWidthStr;

use crate::{
    entry_theme::EntryThemeRef,
    model_state::{ModelState, get_argvs},
    schema::Argument,
    ui::{LEVEL_BASE, on_event_ele},
    ui_content::RenderCtx,
    ui_eleinfo::EleOptions,
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

    fn render_label(
        &self,
        ctx: &mut RenderCtx,
        root: &mut ScrollView,
        area: Rect,
        theme: EntryThemeRef,
    ) {
        let mut label = Line::default();
        label
            .push_span(Span::from(format!(" {}", &self.name)).style(theme.argu_label_name_style()));
        label.push_span(
            Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
        );
        label.push_span(Span::from(self.kind.kind()).style(theme.argu_label_type_style()));
        if self.required.unwrap_or(false) {
            label.push_span(
                Span::from(theme.argu_label_sep_token()).style(theme.argu_label_sep_style()),
            );
            label.push_span(
                Span::from(theme.argu_label_required_token())
                    .style(theme.argu_label_required_style()),
            );
        }

        let mut add_value_idx: Option<usize> = None;
        let mut desc_indicator_idx: Option<usize> = None;

        let mut spans = vec![];
        let mut constraints = vec![ratatui::layout::Constraint::Length(std::cmp::min(
            label.width() as u16,
            60,
        ))];

        let sep = theme.argu_label_sep_token();
        let sep_width = std::cmp::min(sep.width() as u16, 3);

        macro_rules! push_sep {
            () => {
                spans.push(Span::from(sep).style(theme.argu_label_sep_style()));
                constraints.push(ratatui::layout::Constraint::Length(sep_width));
            };
        }

        macro_rules! push_token {
            ($token: expr, $style: expr, $maxl: expr) => {{
                let _token = $token;
                let _width = std::cmp::min(_token.width() as u16, $maxl);
                spans.push(Span::from(_token).style($style));
                constraints.push(ratatui::layout::Constraint::Length(_width));
            }};
        }

        if self.repeatable.unwrap_or(false) {
            push_sep!();
            push_token!(
                theme.argu_label_add_value_token(),
                theme.argu_label_add_value_style(),
                10
            );
            add_value_idx = Some(constraints.len());
        }

        if let Some(description) = self.description.as_ref() {
            let mut show_full_desc = false;

            if !description.is_empty() && !description.contains('\n') {
                let mut test_constraints = constraints.clone();
                test_constraints.push(ratatui::layout::Constraint::Fill(1));
                let test_layouts = Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .constraints(test_constraints)
                    .split(area);
                let desc_remain_width = test_layouts.last().unwrap().width;
                if (description.width() + sep_width as usize) <= desc_remain_width as usize {
                    show_full_desc = true;
                    push_sep!();
                    spans
                        .push(Span::from(description.clone()).style(theme.argu_label_desc_style()));
                    constraints.push(ratatui::layout::Constraint::Fill(1));
                }
            }

            if !show_full_desc {
                push_sep!();
                push_token!(
                    theme.argu_label_desc_indicator_token(),
                    theme.argu_label_desc_indicator_style(),
                    10
                );
                desc_indicator_idx = Some(constraints.len());
            }
        }

        constraints.push(ratatui::layout::Constraint::Fill(1));

        let layouts = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        root.render_widget(label, layouts[0]);

        let mut idx: usize = 1;
        for span in spans {
            root.render_widget(span, layouts[idx]);
            idx += 1;
        }

        let id_prefix = "argu/label/";

        if let Some(idx) = add_value_idx {
            let area = layouts[idx];
            let name = self.name.clone();
            ctx.push(move |eles| {
                on_event_ele(
                    eles,
                    LEVEL_BASE,
                    format!("{id_prefix}/add_val#{}", name),
                    area,
                    Some(
                        EleOptions::default().set_action(crate::ui_eleinfo::ActiveAction::AddArgv),
                    ),
                );
            });
        }
        if let Some(idx) = desc_indicator_idx {
            let area = layouts[idx];
            let name = self.name.clone();
            ctx.push(move |eles| {
                on_event_ele(
                    eles,
                    LEVEL_BASE,
                    format!("{id_prefix}/desc_indicator#{}", &name),
                    area,
                    Some(
                        EleOptions::default()
                            .set_action(crate::ui_eleinfo::ActiveAction::ShowArguDesc(name)),
                    ),
                );
            });
        }
    }

    fn render_input(
        &self,
        ctx: &mut RenderCtx,
        root: &mut ScrollView,
        idx: usize,
        area: Rect,
        val: Option<&str>,
        theme: EntryThemeRef,
        path: &[String],
    ) {
        let input_id = Argument::mk_input_id(&self.name, path, idx);
        root.render_widget(
            Paragraph::new(val.as_ref().map_or("".to_string(), |v| v.to_string()))
                .block(
                    Block::new()
                        .borders(Borders::BOTTOM)
                        .border_style(theme.clone().argu_input_border_style(false)),
                )
                .style(Style::default().bg(ratatui::style::Color::Black)),
            area,
        );
        ctx.push(move |eles| {
            on_event_ele(
                eles,
                LEVEL_BASE,
                format!(""),
                area,
                Some(EleOptions::new(true).set_input_id(&input_id)),
            )
        });
    }

    pub(crate) fn render(
        &self,
        ctx: &mut RenderCtx,
        root: &mut ScrollView,
        area: Rect,
        model_state: &ModelState,
        path: &[String],
        theme: EntryThemeRef,
    ) {
        root.render_widget(
            Block::new()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(theme.clone().argu_wrapper_border_style()),
            area,
        );

        let layouts = Layout::default()
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Fill(1),
            ])
            .split(area);

        self.render_label(ctx, root, layouts[0], theme.clone());

        let area = layouts[1];
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
            constraints.push(ratatui::layout::Constraint::Length(2));
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
                ctx,
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
    pub(crate) fn mk_input_id(argu_name: &str, path: &[String], idx: usize) -> String {
        return format!("{}:{}#{}", path.join("/"), argu_name, idx);
    }
}
