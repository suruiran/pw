use std::{cell::RefCell, rc::Rc};

use crossterm::event::Event;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Layout, Rect, Size},
    widgets::{Paragraph, Widget},
};
use tui_scrollview::ScrollViewState;

use crate::{
    entry_theme::EntryThemeRef,
    model_state::ModelState,
    repl::repl,
    schema::{self, Argument},
    tui::{
        content::RenderCtx,
        eleinfo::{EleIndex, EleOptions, Element},
        layers::{EleLevel, UILayersRef},
    },
};

#[derive(Debug, Default)]
pub(crate) struct ScrollViewInfo {
    pub(crate) area: Rect,
    pub(crate) size: Size,
    pub(crate) state: ScrollViewState,
}

pub(crate) type ScrollViewInfoRef = Rc<RefCell<Option<ScrollViewInfo>>>;

pub(crate) struct UIApp {
    pub(crate) cmd: Rc<schema::Command>,
    pub(crate) model_state: Rc<RefCell<ModelState>>,
    pub(crate) theme: EntryThemeRef,

    pub(crate) focused: Option<(i32, usize)>,

    pub(crate) mouse_enabled: bool,

    pub(crate) renderctx: Rc<RefCell<RenderCtx>>,
    pub(crate) layers: UILayersRef,
    pub(crate) scrollview: ScrollViewInfoRef,
    pub(crate) prev_path: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(crate) struct UIState {
    pub(crate) path: Vec<String>,
    pub(crate) arg: Option<String>,
}

impl UIApp {
    pub(crate) fn current_ui_state(&mut self) -> UIState {
        let mut current_cmd: &schema::Command = &self.cmd;
        let mut path: Vec<String> = vec![current_cmd.exe.to_string()];
        let mut found_cmd: bool = false;

        let model_state = self.model_state.borrow();

        if let Some(cidx) = model_state.current {
            if cidx >= model_state.stack.len() {
                current_cmd = &self.cmd;
            } else {
                for idx in 1..=cidx {
                    let cmdv = &model_state.stack[idx];
                    if current_cmd.subs.is_none() {
                        break;
                    }
                    match current_cmd.subs.as_ref().unwrap().get(&cmdv.name) {
                        Some(v) => {
                            current_cmd = v;
                            path.push(cmdv.name.clone());
                        }
                        None => {
                            break;
                        }
                    }
                }
                found_cmd = true;
            }
        }
        drop(model_state);

        if !found_cmd {
            let mut model_state = self.model_state.borrow_mut();
            model_state.stack = vec![Default::default()];
            model_state.current = Some(0);

            path = vec![current_cmd.exe.to_string()];
        }

        let model_state = self.model_state.borrow();

        let mut current_argv_name: Option<String> = None;
        let current_cmd_with_val = &model_state.stack[model_state.current.unwrap()];
        match current_cmd_with_val.current_argu.as_ref() {
            Some(v) => {
                current_argv_name = Some(v.clone());
            }
            None => match current_cmd.args.as_ref() {
                Some(args) => {
                    if !args.is_empty() {
                        current_argv_name = Some((&args[0]).name.clone())
                    }
                }
                _ => {}
            },
        }

        let inputid = Argument::mk_input_id(
            current_argv_name.as_ref().unwrap_or(&"".to_string()),
            &path,
            current_cmd_with_val.current_argv.unwrap_or(0),
        );

        if inputid != model_state.inputid {
            drop(model_state);
            let mut model_state = self.model_state.borrow_mut();

            model_state.inputid.clear();
            model_state.inputtemp.clear();
        }

        return UIState {
            path: path,
            arg: current_argv_name.clone(),
        };
    }
}

pub(crate) fn get_current_schema<'a>(
    root: &'a Rc<schema::Command>,
    path: &Vec<String>,
) -> &'a schema::Command {
    if path.len() == 1 {
        return root.as_ref();
    }
    let mut cmd = root.as_ref();
    for name in path.iter().skip(1) {
        cmd = cmd.subs.as_ref().unwrap().get(name).unwrap();
    }
    return cmd;
}

// render
impl UIApp {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut changed = true;
        loop {
            if changed {
                self.layers.borrow_mut().clear();

                let state = Rc::new(self.current_ui_state());
                self.prev_path = Some(state.path.clone());
                terminal.draw(|frame| self.render(frame, state.clone()))?;
            }

            match self.react(crossterm::event::read()?, terminal.size()?) {
                super::event::EvtReturn::Ignore => {
                    changed = false;
                    continue;
                }
                super::event::EvtReturn::Ok => {
                    changed = true;
                    continue;
                }
                super::event::EvtReturn::Exit => {
                    return Ok(());
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, uistate: Rc<UIState>) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_content(layout[0], uistate.clone());
        self.render_footer(layout[1], uistate.clone());
        self.layers.borrow_mut().render(frame);
        self.renderctx.borrow_mut().drain(self.layers.clone());
    }

    fn render_footer(&mut self, container: Rect, uistate: Rc<UIState>) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                // cmd path
                ratatui::layout::Constraint::Max(40),
                ratatui::layout::Constraint::Fill(1),
                // btn group
                ratatui::layout::Constraint::Length(20),
            ])
            .split(container);

        self.render_footer_path(layout[0], uistate.clone());
        self.render_footer_btns(layout[2], uistate.clone());
    }

    fn render_footer_path(&mut self, container: Rect, uistate: Rc<UIState>) {
        let mut constraints = vec![];
        for _ in 0..uistate.path.len() {
            constraints.push(ratatui::layout::Constraint::Fill(1));
        }
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(constraints)
            .split(container);

        for (idx, name) in uistate.path.iter().enumerate() {
            let id = format!("footer/path/{}", idx);
            self.on_plain_ele(
                EleLevel::Base,
                id.clone(),
                Paragraph::new(name.clone()),
                layout[idx],
                None,
            );
        }
    }

    fn render_footer_btns(&mut self, container: Rect, uistate: Rc<UIState>) {}

    pub fn on_state_ele(
        &mut self,
        level: EleLevel,
        id: String,
        render: Box<dyn FnOnce(&mut Frame, Rect)>,
        area: Rect,
        opts: Option<EleOptions>,
    ) {
        let eletemp = Element {
            index: EleIndex::default(),
            id,
            render_fn: Some(render),
            area,
            opts,
        };
        on_ele(self.layers.clone(), level, eletemp);
    }

    pub fn on_plain_ele<W: Widget + 'static>(
        &mut self,
        level: EleLevel,
        id: String,
        widget: W,
        area: Rect,
        opts: Option<EleOptions>,
    ) {
        self.on_state_ele(
            level,
            id,
            Box::new(move |f, a| {
                f.render_widget(widget, a);
            }),
            area,
            opts,
        );
    }
}

impl Drop for UIApp {
    fn drop(&mut self) {
        ratatui::restore();
        if self.mouse_enabled {
            _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        }
    }
}

pub fn run(cmd: schema::Command) -> Result<Vec<String>, String> {
    if cmd.is_empty() {
        return Ok(vec![cmd.exe]);
    }
    match color_eyre::install() {
        Ok(_) => {}
        Err(e) => {
            return Err(e.to_string());
        }
    }

    let mut terminal = ratatui::init();
    match terminal.size() {
        Ok(size) => {
            if size.width < 60 || size.height < 15 {
                ratatui::restore();
                return repl(cmd);
            }
        }
        Err(err) => {
            return Err(format!("read terminal size failed: {}", err.to_string()));
        }
    }

    let mut app = UIApp {
        cmd: Rc::new(cmd),
        model_state: Default::default(),
        mouse_enabled: crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
            .is_ok(),
        renderctx: Default::default(),
        layers: Default::default(),
        scrollview: Default::default(),
        prev_path: None,
        theme: Default::default(),
        focused: Default::default(),
    };
    match app.run(&mut terminal) {
        Ok(_) => {}
        Err(e) => {
            return Err(e.to_string());
        }
    }
    return Ok(vec![]);
}

pub(crate) fn on_event_ele(
    eles: UILayersRef,
    level: EleLevel,
    id: String,
    area: Rect,
    opts: Option<EleOptions>,
) {
    let eletemp = Element {
        id,
        render_fn: None,
        area,
        opts,
        index: EleIndex::default(),
    };
    on_ele(eles, level, eletemp);
}

fn on_ele(eles: UILayersRef, level: EleLevel, ele: Element) {
    let mut eles = eles.borrow_mut();
    eles.push(level, ele);
}
