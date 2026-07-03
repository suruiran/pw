use crossterm::style::Stylize;
use std::rc::Rc;
use unicode_width::UnicodeWidthStr;

use ratatui::{
    layout::{Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_scrollview::ScrollView;

use crate::{
    entry_theme::{EntryTheme, EntryThemeRef},
    model_state::{ModelState, get_argv},
    schema::Argument,
    ui_style::style_by_id,
};

impl Argument {
    pub(crate) fn height(&self, model_state: &ModelState, path: &[String]) -> u16 {
        let mut height: u16 = 5;
        let mut vsize: u16 = 1;
        if self.repeatable.unwrap_or(false) {
            if let Some(argv) = get_argv(model_state, path, &self.name) {
                vsize = argv.len() as u16;
            }
            height += 1;
        }
        if vsize < 1 {
            vsize = 1;
        }
        if vsize > 1 {
            height += (vsize - 1) * 2;
        }
        return height;
    }

    fn render_label(&self, root: &mut ScrollView, pid: &str, area: Rect, theme: EntryThemeRef) {
        let mut constraints = vec![
            // name
            ratatui::layout::Constraint::Max(20),
            // type
            ratatui::layout::Constraint::Max(10),
        ];
        let mut remain_width = area.width as i32 - 30;

        if self.required.unwrap_or(false) {
            constraints.push(ratatui::layout::Constraint::Length(1));
            remain_width -= 1
        } else {
            constraints.push(ratatui::layout::Constraint::Length(0));
        }
        let mut desc: Option<&str> = None;
        let mut show_full_desc = false;
        if let Some(d) = self.description.as_ref() {
            let single_line = !d.contains('\n');
            let mut desc_width = 2 as i32;
            if single_line {
                desc_width = d.width() as i32;
            }
            if d.len() > 0 {
                desc = Some(d);
                if single_line && desc_width <= remain_width {
                    show_full_desc = true;
                } else {
                    show_full_desc = false;
                    desc_width = 2
                }
                constraints.push(ratatui::layout::Constraint::Length(desc_width as u16));
            }
        }
        if desc.is_none() {
            constraints.push(ratatui::layout::Constraint::Length(0));
        }

        constraints.push(ratatui::layout::Constraint::Fill(1));

        let layouts = Layout::new(ratatui::layout::Direction::Horizontal, constraints).split(area);

        // name
        root.render_widget(
            Paragraph::new(self.name.clone()).style(style_by_id(
                theme.clone(),
                format!("{pid}/label/name:{}", &self.name).as_str(),
            )),
            layouts[0],
        );

        // type
        root.render_widget(
            Paragraph::new(self.kind.kind()).style(style_by_id(
                theme.clone(),
                format!("{pid}/label/type:{}", &self.name).as_str(),
            )),
            layouts[1],
        );

        // required
        if self.required.unwrap_or(false) {
            root.render_widget(
                Paragraph::new("*").style(style_by_id(
                    theme.clone(),
                    format!("{pid}/label/required:{}", &self.name).as_str(),
                )),
                layouts[2],
            );
        }

        // desc
        if let Some(desc) = desc {
            if show_full_desc {
                root.render_widget(
                    Paragraph::new(desc.to_string()).style(style_by_id(
                        theme.clone(),
                        format!("{pid}/label/desc:{}", &self.name).as_str(),
                    )),
                    layouts[3],
                );
            } else {
                root.render_widget(
                    Paragraph::new("🙋‍♂️").style(style_by_id(
                        theme.clone(),
                        format!("{pid}/label/desc_indicator:{}", &self.name).as_str(),
                    )),
                    layouts[3],
                );
            }
        }
    }

    fn render_input(
        &self,
        root: &mut ScrollView,
        pid: &str,
        idx: usize,
        area: Rect,
        val: Option<&str>,
        theme: EntryThemeRef,
        path: &[String],
    ) {
        let input_id = Argument::mk_input_id(self, path);
        tracing::warn!(">>>> {input_id} {:?}", area);
        root.render_widget(
            Paragraph::new(val.as_ref().map_or("".to_string(), |v| v.to_string()))
                .style(Style::new().fg(ratatui::style::Color::Yellow))
                .block(
                    Block::bordered()
                        .title(self.name.as_str())
                        .title_style(Style::new().fg(ratatui::style::Color::Green)),
                ),
            area,
        );
    }

    fn render_values(
        &self,
        root: &mut ScrollView,
        pid: &str,
        area: Rect,
        model_state: &ModelState,
        path: &[String],
        theme: EntryThemeRef,
    ) {
        let mut constraints = vec![];

        let argv = get_argv(model_state, path, &self.name);
        let value_count = argv.as_ref().map_or_else(|| 0, |v| v.len());
        let mut input_count = 1;
        if self.repeatable.unwrap_or(false) {
            input_count = value_count + 1;
            if input_count < 2 {
                input_count = 2;
            }
        }
        for _ in 0..input_count {
            constraints.push(ratatui::layout::Constraint::Length(3));
        }

        tracing::info!("ArgArea: {} {:?}", &self.name, area);

        let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints).split(area);
        let mut lidx: usize = 0;

        for vidx in 0..input_count - 1 {
            self.render_input(
                root,
                format!("{pid}/input").as_str(),
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

    pub(crate) fn render(
        &self,
        root: &mut ScrollView,
        area: Rect,
        model_state: &ModelState,
        path: &[String],
        theme: EntryThemeRef,
    ) {
        let layouts = Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![
                // label
                ratatui::layout::Constraint::Length(1),
                // values
                ratatui::layout::Constraint::Fill(1),
            ],
        )
        .split(area);

        let ele_id_prefix = "content/scrollview/argu";
        self.render_label(root, ele_id_prefix, layouts[0], theme.clone());
        self.render_values(
            root,
            ele_id_prefix,
            layouts[1],
            model_state,
            path,
            theme.clone(),
        );
    }
}

// static methods
impl Argument {
    pub(crate) fn mk_input_id(argv: &Argument, path: &[String]) -> String {
        return format!("{}:{}", path.join("/"), &argv.name);
    }
}
