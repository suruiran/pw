use ratatui::{Frame, layout::Size};

use crate::tui::{
    app::{ScrollViewInfo, TUIApp},
    element::{EleIndex, Element},
    layers::UILayers,
};

#[derive(Default)]
pub(crate) struct RenderCtx {
    focused: Option<EleIndex>,
    layers: UILayers,
    scrollview: Option<ScrollViewInfo>,
}

impl RenderCtx {
    pub(crate) fn push(&mut self, ele: Element) -> &mut Self {
        self.layers.push(ele);
        return self;
    }

    pub(crate) fn clear(&mut self) {
        self.layers.clear();
    }

    pub(crate) fn render(&mut self, frame: &mut Frame) {
        self.layers.render(frame, &self.scrollview);
    }

    pub(crate) fn with_scrollview_mut<R>(
        &mut self,
        f: impl FnOnce(&mut Option<ScrollViewInfo>) -> R,
    ) -> R {
        return f(&mut self.scrollview);
    }

    pub(crate) fn with_focusables<R>(&self, vpsize: Size, f: impl FnOnce(Vec<&Element>) -> R) -> R {
        let eles = self.layers.all_focusable(vpsize);
        return f(eles);
    }

    pub(crate) fn auto_focused(&mut self, vpsize: Size) -> bool {
        if self.focused.is_some() {
            return false;
        }
        let mut idx: Option<EleIndex> = None;
        let ele = self
            .layers
            .all_focusable(vpsize)
            .into_iter()
            .find(|v| v.opts.is_some() && v.opts.as_ref().unwrap().auto_focusable);
        if let Some(ele) = ele {
            idx = Some(ele.index.clone());
        }
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
                let ele = self.layers.get(idx);
                return Some(f(ele));
            }
            None => None,
        }
    }
}
