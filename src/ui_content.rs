use std::{cell::RefCell, rc::Rc};

use ratatui::layout::{Layout, Rect, Size};
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::ui::{LEVEL_BASE, UIApp, UIState, get_current_schema};

impl UIApp {
    pub(crate) fn render_content(&mut self, container: Rect, uistate: Rc<UIState>) {
        let wrapper_id = "content/scrollview".to_string();

        if let Some(prev_path) = self.prev_path.as_ref()
            && prev_path.join("/") != uistate.path.join("/")
        {
            self.states.remove(&wrapper_id);
        }

        let root = self.cmd.clone();
        let schema = get_current_schema(&root, &uistate.path);

        const ELE_HEIGHT: u16 = 2;

        let mut height = 0;
        if schema.has_args() {
            height += schema.args.as_ref().unwrap().len() * (ELE_HEIGHT as usize);
        }
        if schema.has_subs() {
            height += ELE_HEIGHT as usize;
        }

        {
            let size = Size::new(container.width, height as u16);
            // hold memory
            let scrollview_state = self
                .states
                .entry(wrapper_id.clone())
                .or_insert(Rc::new(RefCell::new(ScrollViewState::new())))
                .clone();
            let root = self.cmd.clone();
            let uistate = uistate.clone();
            let theme = self.theme.clone();
            let model_state = self.state.clone();
            self.on_state_ele(
                LEVEL_BASE,
                wrapper_id.clone(),
                Box::new(move |f, a| {
                    let schema = get_current_schema(&root, &uistate.path);
                    let scrollview_state = scrollview_state
                        .borrow_mut()
                        .downcast_mut::<ScrollViewState>()
                        .expect("unexpect state type, required ScrollViewState");

                    let mut scrollview = Rc::new(RefCell::new(ScrollView::new(size)));

                    let mut constraints = vec![];
                    if schema.has_args() {
                        for argv in schema.args.as_ref().unwrap() {
                            let mut ele_len = ELE_HEIGHT;
                            if argv.repeatable.unwrap_or(false) {}
                            constraints.push(ratatui::layout::Constraint::Length(ELE_HEIGHT));
                        }
                    }
                    if schema.has_subs() {
                        constraints.push(ratatui::layout::Constraint::Length(ELE_HEIGHT));
                    }

                    let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints)
                        .split(scrollview.borrow().area());

                    let mut lidx = 0;
                    if schema.has_args() {
                        for argv in schema.args.as_ref().unwrap() {
                            let argv_layouts = Layout::default()
                                .direction(ratatui::layout::Direction::Vertical)
                                .constraints(vec![
                                    ratatui::layout::Constraint::Fill(1),
                                    ratatui::layout::Constraint::Fill(1),
                                ])
                                .split(layouts[lidx]);
                            lidx += 1;

                            let argv_id = format!(
                                "{}/argv/{}/label",
                                wrapper_id.as_str(),
                                argv.name.as_str()
                            );
                            let scrollview = scrollview.clone();
                            argv.label(&argv_id, argv_layouts[0], move |p, a| {
                                let mut scrollview = scrollview.borrow_mut();
                                scrollview.render_widget(p, a);
                            });
                        }
                    }
                }),
                container,
            );
        }
    }
}
