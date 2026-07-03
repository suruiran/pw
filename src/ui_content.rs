use std::rc::Rc;

use ratatui::layout::{Layout, Margin, Rect, Size};
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::ui::{LEVEL_BASE, UIApp, UIState, get_current_schema};

impl UIApp {
    pub(crate) fn render_content(&mut self, container: Rect, uistate: Rc<UIState>) {
        let container = container.inner(Margin::new(1, 0));

        let wrapper_id = "content/scrollview".to_string();

        if let Some(prev_path) = self.prev_path.as_ref()
            && prev_path.join("/") != uistate.path.join("/")
        {
            let mut scrollview_state = self.scrollview_sate.borrow_mut();
            *scrollview_state = None;
        }

        {
            let mut scrollview_state = self.scrollview_sate.borrow_mut();
            if scrollview_state.is_none() {
                *scrollview_state = Some(ScrollViewState::new());
            }
        }

        let scrollview_state = self.scrollview_sate.clone();

        let root = self.cmd.clone();
        let uistate = uistate.clone();
        let model_state = self.model_state.clone();
        let theme = self.theme.clone();
        self.on_state_ele(
            LEVEL_BASE,
            wrapper_id.clone(),
            Box::new(move |f, a: Rect| {
                let schema = get_current_schema(&root, &uistate.path);
                let args = schema.available_args(&model_state.borrow(), &uistate.path);

                let mut constraints = vec![];
                let mut height = 0;
                if args.len() > 0 {
                    for arg in args.iter() {
                        let arg_height = arg.height(&model_state.borrow(), &uistate.path);
                        height += arg_height;
                        constraints.push(ratatui::layout::Constraint::Length(arg_height));
                    }
                }
                if schema.has_subs() {
                    height += 2;
                    constraints.push(ratatui::layout::Constraint::Length(2));
                }
                let size = Size::new(container.width, height as u16);

                let mut scrollview_state = scrollview_state.borrow_mut();
                let scrollview_state = scrollview_state
                    .as_mut()
                    .expect("unreachable code: empty ScrollViewState");
                let mut scrollview = ScrollView::new(size)
                    .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);

                let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints).split(
                    scrollview.area().inner(Margin {
                        horizontal: 1,
                        vertical: 0,
                    }),
                );

                {
                    let model_state = model_state.borrow();
                }

                let mut lidx = 0;
                for arg in args.iter() {
                    arg.render(
                        &mut scrollview,
                        layouts[lidx],
                        &model_state.borrow(),
                        &uistate.path,
                        theme.clone(),
                    );
                    lidx += 1;
                }

                f.render_stateful_widget(scrollview, a, scrollview_state);
            }),
            container,
        );
    }
}
