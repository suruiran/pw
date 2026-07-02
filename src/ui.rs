use std::{cell::RefCell, collections::HashMap, ops::Deref};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{repl::repl, schema};

#[derive(Debug, Default)]
struct ArgvWithValue {
    name: String,
    value: Option<Vec<String>>,
}

#[derive(Debug, Default)]
struct CmdWithValue {
    name: String,
    args: Vec<ArgvWithValue>,
    current: Option<String>,
}

#[derive(Debug, Default)]
struct State {
    cmds: Vec<CmdWithValue>,
    current: Option<usize>,

    inputid: String,
    inputtemp: String,
}

struct EleTempInfo {
    id: String,
    redner: Option<Box<dyn FnOnce(&mut Frame, Rect)>>,
    area: Rect,
}

const LEVEL_BASE: i32 = 0;
const LEVEL_FLOATING: i32 = 10000;
const LEVEL_NOTIFY: i32 = 20000;

struct App {
    cmd: Box<schema::Command>,
    state: State,

    evt: Option<Event>,

    mouse_enabled: bool,

    eletemps: HashMap<i32, Vec<EleTempInfo>>,
}

#[derive(Debug)]
struct UIState {
    path: Vec<String>,
    arg: Option<String>,
}

impl App {
    fn current_ui_state(&mut self) -> UIState {
        let mut current_cmd: &schema::Command = &self.cmd;
        let mut path: Vec<String> = vec![current_cmd.exe.to_string()];
        let mut found_cmd: bool = false;

        if let Some(cidx) = self.state.current {
            if cidx >= self.state.cmds.len() {
                current_cmd = &self.cmd;
            } else {
                for idx in 1..=cidx {
                    let cmdv = &self.state.cmds[idx];
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
        if !found_cmd {
            self.state.cmds = vec![Default::default()];
            self.state.current = Some(0);

            path = vec![current_cmd.exe.to_string()];
        }

        let mut current_argv_name: Option<String> = None;

        let current_cmd_with_val = &self.state.cmds[self.state.current.unwrap()];
        match current_cmd_with_val.current.as_ref() {
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

        let inputid = format!(
            "{}::{}",
            path.join("."),
            current_argv_name.as_deref().unwrap_or("")
        );

        if inputid != self.state.inputid {
            self.state.inputid = inputid;
            self.state.inputtemp.clear();
        }

        return UIState {
            path: path,
            arg: current_argv_name.clone(),
        };
    }

    // Must call this method after `current_ui_state`, make sure path is validate.
    fn current_schema(&self, path: &Vec<String>) -> &schema::Command {
        if path.len() == 1 {
            return &self.cmd;
        }
        let mut cmd = self.cmd.as_ref();
        for name in path.iter().skip(1) {
            cmd = cmd.subs.as_ref().unwrap().get(name).unwrap();
        }
        return cmd;
    }
}

// render
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            let state = self.current_ui_state();
            self.eletemps.clear();
            terminal.draw(|frame| self.render(frame, &state))?;
            self.evt = Some(crossterm::event::read()?);
            if !self.react() {
                return Ok(());
            }
        }
    }

    fn _on_key_evt(&mut self, evt: KeyEvent) -> bool {
        if evt.is_release() {
            return true;
        }

        tracing::info!("{:?}", evt);

        match evt.code {
            KeyCode::Esc => {
                return false;
            }
            KeyCode::Char(code) => {
                if evt.modifiers.contains(KeyModifiers::CONTROL) {
                    match code {
                        'c' | 'z' => {
                            return false;
                        }
                        _ => {}
                    }
                }
                return true;
            }
            _ => {
                return true;
            }
        }
    }

    fn _on_mouse_ele(&mut self, evt: MouseEvent, id: String) -> bool {
        return true;
    }

    fn _on_mouse_evt(&mut self, evt: MouseEvent) -> bool {
        return true;
    }

    fn react(&mut self) -> bool {
        if let Some(Event::Key(keyevt)) = self.evt {
            return self._on_key_evt(keyevt);
        }
        if self.mouse_enabled
            && let Some(Event::Mouse(mouseevt)) = self.evt
        {
            return self._on_mouse_evt(mouseevt);
        }
        return true;
    }

    fn render(&mut self, frame: &mut Frame, uistate: &UIState) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_content(frame, layout[0], uistate);
        self.render_footer(frame, layout[1], uistate);

        self._render(frame);
    }

    fn render_content(&mut self, frame: &mut Frame, container: Rect, uistate: &UIState) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(container);

        frame.render_widget("xxxx", container);
    }

    fn render_footer(&mut self, frame: &mut Frame, container: Rect, uistate: &UIState) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                // cmd path
                ratatui::layout::Constraint::Length(40),
                ratatui::layout::Constraint::Fill(1),
                // btn group
                ratatui::layout::Constraint::Length(20),
            ])
            .split(container);

        self.render_footer_path(frame, layout[0], uistate);
        self.render_footer_btns(frame, layout[2], uistate);
    }

    fn render_footer_path(&mut self, frame: &mut Frame, container: Rect, uistate: &UIState) {
        let mut constraints = vec![];
        for _ in 0..uistate.path.len() {
            constraints.push(ratatui::layout::Constraint::Fill(1));
        }
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(constraints)
            .split(container);

        for (idx, name) in uistate.path.iter().enumerate() {
            self.on_ele(
                LEVEL_BASE,
                format!("footer/path/{}", idx),
                Paragraph::new(name.clone()).style(Style::new().fg(Color::Green).underlined()),
                layout[idx],
            );
        }
    }

    fn render_footer_btns(&mut self, frame: &mut Frame, container: Rect, uistate: &UIState) {}

    fn on_ele<W: Widget + 'static>(&mut self, level: i32, id: String, widget: W, area: Rect) {
        let leveleles = self.eletemps.entry(level).or_insert(vec![]);
        let eletemp = EleTempInfo {
            id,
            redner: Some(Box::new(move |f, a| {
                f.render_widget(widget, a);
            })),
            area,
        };
        leveleles.push(eletemp);
    }

    fn _render(&mut self, frame: &mut Frame) {
        let mut levels = self.eletemps.keys().map(|v| *v).collect::<Vec<_>>();
        levels.sort();

        for level in levels.iter() {
            if let Some(eles) = self.eletemps.get_mut(level) {
                for ele in eles {
                    if let Some(rf) = ele.redner.take() {
                        rf(frame, ele.area);
                    }
                }
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
        if self.mouse_enabled {
            _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        }
    }
}

pub(crate) fn ui(cmd: schema::Command) -> Result<Vec<String>, String> {
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

    let mut app = App {
        cmd: Box::new(cmd),
        state: Default::default(),
        evt: None,
        mouse_enabled: crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
            .is_ok(),
        eletemps: Default::default(),
    };
    match app.run(&mut terminal) {
        Ok(_) => {}
        Err(e) => {
            return Err(e.to_string());
        }
    }
    return Ok(vec![]);
}
