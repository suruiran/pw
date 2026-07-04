use std::rc::Rc;

use ratatui::layout::{Layout, Rect, Size};
use tui_scrollview::ScrollView;

use crate::tui::{
    app::{ScrollViewInfo, UIApp, UIState, get_current_schema},
    consts::uiconsts,
    layers::{EleLevel, UILayersRef},
};

impl UIApp {
    pub(crate) fn render_content(&mut self, container: Rect, uistate: Rc<UIState>) {
        let container = container.inner(uiconsts::MARGIN);

        let wrapper_id = "content/scrollview".to_string();

        if let Some(prev_path) = self.prev_path.as_ref()
            && prev_path.join("/") != uistate.path.join("/")
        {
            let mut scrollview = self.scrollview.borrow_mut();
            *scrollview = None;
        }

        {
            let mut scrollview = self.scrollview.borrow_mut();
            if scrollview.is_none() {
                let mut val = ScrollViewInfo::default();
                val.area = container;

                *scrollview = Some(val);
            }
        }

        let scrollview = self.scrollview.clone();

        let root = self.cmd.clone();
        let uistate = uistate.clone();
        let model_state = self.model_state.clone();
        let theme = self.theme.clone();

        let ctx = self.renderctx.clone();
        self.on_state_ele(
            EleLevel::Base,
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
                    constraints.push(ratatui::layout::Constraint::Length(
                        uiconsts::ARGU_INPUT_HEIGHT,
                    ));
                }
                let size = Size::new(container.width, height as u16);

                let mut scrollview = scrollview.borrow_mut();
                let scrollview = scrollview
                    .as_mut()
                    .expect("unreachable code: empty ScrollViewInfo");
                scrollview.size = size;
                let scrollview_state = &mut scrollview.state;
                let mut scrollview = ScrollView::new(size)
                    .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);

                let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints)
                    .split(scrollview.area().inner(uiconsts::MARGIN));

                let mut ctx = ctx.borrow_mut();

                let mut lidx = 0;
                for arg in args.iter() {
                    arg.render(
                        &mut ctx,
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
            None,
        );
    }
}

#[derive(Default)]
pub(crate) struct RenderCtx {
    lazys: Vec<Box<dyn FnOnce(UILayersRef) + 'static>>,
}

impl RenderCtx {
    pub(crate) fn push<F: FnOnce(UILayersRef) + 'static>(&mut self, f: F) -> &mut Self {
        self.lazys.push(Box::new(f));
        return self;
    }

    pub(crate) fn drain(&mut self, eles: UILayersRef) {
        for f in std::mem::take(&mut self.lazys) {
            f(eles.clone());
        }
    }
}
