use std::rc::Rc;

use ratatui::layout::{Layout, Rect, Size};
use tui_scrollview::ScrollView;

use crate::tui::{
    app::{ScrollViewInfo, TUIApp, UIState, get_current_schema},
    consts::uiconsts,
    element::Element,
    layers::EleLevel,
};

impl TUIApp {
    pub(crate) fn render_content(&mut self, container: Rect, uistate: Rc<UIState>) {
        let container = container.inner(uiconsts::MARGIN);
        if let Some(prev_path) = self.prev_path.as_ref()
            && prev_path.join("/") != uistate.path.join("/")
        {
            self.ctx.with_scrollview_mut(|sv| {
                *sv = None;
            });
        }

        self.ctx.with_scrollview_mut(|sv| {
            if sv.is_some() {
                return;
            }
            let mut val = ScrollViewInfo::default();
            val.area = container;
            *sv = Some(val);
        });

        let schema = get_current_schema(&self.cmd, &uistate.path);
        let argus = schema.available_args(&self.model_state, &uistate.path);

        let mut constraints = vec![];
        let mut height = 0;
        if argus.len() > 0 {
            for argu in argus.iter() {
                let arg_height = argu.height(&self.model_state, &uistate.path);
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

        let mut scrollview = ScrollView::new(size)
            .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);

        let layouts = Layout::new(ratatui::layout::Direction::Vertical, constraints)
            .split(scrollview.area().inner(uiconsts::MARGIN));

        let ctx = &mut self.ctx;

        let mut lidx = 0;
        for arg in argus.iter() {
            arg.render(
                ctx,
                &mut scrollview,
                layouts[lidx],
                &self.model_state,
                &uistate.path,
                self.theme.clone(),
            );
            lidx += 1;
        }

        let mut ele: Option<Element> = None;
        ctx.with_scrollview_mut(|sv| {
            let scrollview_info = sv.as_mut().expect("");
            scrollview_info.size = size;

            ele = Some(
                Element::new(EleLevel::Base, "content/scrollview", container, None).stateful(
                    scrollview,
                    &mut scrollview_info.state,
                    false,
                ),
            );
        });
        if let Some(ele) = ele {
            ctx.push(ele);
        };
    }
}
