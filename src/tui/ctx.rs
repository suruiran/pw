use std::{cell::RefCell, process::id, rc::Rc};

use ratatui::layout::Size;

use crate::tui::{
    app::TUIApp,
    element::{EleIndex, Element},
    layers::UILayersRef,
};

#[derive(Default)]
pub(crate) struct RenderCtx {
    lazys: Vec<Element>,
    focused: Option<EleIndex>,
    pub(crate) layers: Option<UILayersRef>,
}

impl RenderCtx {
    pub(crate) fn push(&mut self, ele: Element) -> &mut Self {
        self.lazys.push(ele);
        return self;
    }

    pub(crate) fn drain(&mut self, eles: UILayersRef) {
        let mut layers = eles.borrow_mut();
        for ele in std::mem::take(&mut self.lazys) {
            layers.push(ele);
        }
    }

    pub(crate) fn auto_focused(&mut self, app: &TUIApp, vpsize: Size) -> bool {
        if self.focused.is_some() {
            return false;
        }
        let mut idx: Option<EleIndex> = None;
        app.with_focusable(vpsize, |eles| {
            let ele = eles
                .into_iter()
                .find(|v| v.opts.is_some() && v.opts.as_ref().unwrap().auto_focusable);
            if let Some(ele) = ele {
                idx = Some(ele.index.clone());
            }
        });
        match idx {
            Some(idx) => {
                tracing::info!("{:?}", &idx);
                self.focused = Some(idx);
                return true;
            }
            None => {
                return false;
            }
        }
    }

    pub(crate) fn with_current_focused_ele<R>(
        &self,
        f: impl FnOnce(Option<&Element>) -> R,
    ) -> Option<R> {
        match self.focused.as_ref() {
            Some(idx) => {
                let layers = self
                    .layers
                    .as_ref()
                    .expect("unreachable code: empty RenderCtx.layers")
                    .borrow();
                let ele = layers.get(idx);
                return Some(f(ele));
            }
            None => None,
        }
    }
}
