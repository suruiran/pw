use std::{cell::RefCell, rc::Rc};

use ratatui::{
    Frame,
    layout::{Rect, Size},
};

use crate::tui::{
    app::ScrollViewInfoRef,
    element::{EleIndex, Element},
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum EleLevel {
    Base,
    Floating,
    Notify,
}

impl Default for EleLevel {
    fn default() -> Self {
        return Self::Base;
    }
}

#[derive(Debug, Default)]
pub(crate) struct UILayers {
    base: Vec<Element>,
    floating: Vec<Element>,
    notify: Vec<Element>,
    adjusted: bool,
}

pub(crate) type UILayersRef = Rc<RefCell<UILayers>>;

impl UILayers {
    pub(crate) fn clear(&mut self) {
        self.base.clear();
        self.floating.clear();
        self.notify.clear();
        self.adjusted = false;
    }

    pub(crate) fn push(&mut self, ele: Element) {
        let eles = match ele.index.level {
            EleLevel::Base => &mut self.base,
            EleLevel::Floating => &mut self.floating,
            EleLevel::Notify => &mut self.notify,
        };

        let mut ele = ele;
        ele.index.idx = eles.len();
        eles.push(ele);
    }

    pub(crate) fn render(&mut self, frame: &mut Frame) {
        macro_rules! call {
            ($eles: expr) => {
                for ele in $eles.iter_mut() {
                    if let Some(rf) = ele.render_fn.take() {
                        rf(frame, ele.area);
                    }
                }
            };
        }
        call!(self.base);
        call!(self.floating);
        call!(self.notify);
    }
}

impl UILayers {
    pub(crate) fn top_level(&self) -> EleLevel {
        if !self.notify.is_empty() {
            return EleLevel::Notify;
        }
        if !self.floating.is_empty() {
            return EleLevel::Floating;
        }
        return EleLevel::Base;
    }

    fn to_refs<F: Fn(&Element) -> bool>(src: &Vec<Element>, ef: F) -> Vec<&Element> {
        let mut refs: Vec<&Element> = src
            .iter()
            .filter(|ele| ele.responsive() && ef(ele))
            .collect();
        refs.sort_by(|a, b| {
            a.area
                .y
                .cmp(&b.area.y)
                .then_with(|| a.area.x.cmp(&b.area.x))
        });
        return refs;
    }

    pub(crate) fn adjust_base_layer(&mut self, scrollview: ScrollViewInfoRef) {
        if self.top_level() != EleLevel::Base {
            return;
        }

        if self.adjusted {
            return;
        }
        self.adjusted = true;

        let scrollviewinfo = scrollview.borrow();
        if scrollviewinfo.is_none() {
            return;
        }
        let scrollviewinfo = scrollviewinfo.as_ref().unwrap();

        let pos = scrollviewinfo.area.as_position();
        let offset = scrollviewinfo.state.offset();

        self.base.iter_mut().for_each(|ele| {
            if let Some(opts) = ele.opts.as_ref() {
                if !opts.in_scroll_view {
                    return;
                }
            } else {
                return;
            }

            let x = ele.area.x as i32 + pos.x as i32 - offset.x as i32;
            if x < 0 || x > u16::MAX as i32 {
                ele.area.width = 0;
                return;
            }
            let y = ele.area.y as i32 + pos.y as i32 - offset.y as i32;
            if y < 0 || y > u16::MAX as i32 {
                ele.area.width = 0;
                return;
            }
            ele.area.x = x as u16;
            ele.area.y = y as u16;
        });
    }

    pub(crate) fn all_focusable(&self, vpsize: Size) -> Vec<&Element> {
        match self.top_level() {
            EleLevel::Base => {
                let viewport = Rect::from(vpsize);
                return Self::to_refs(&self.base, move |e| viewport.intersects(e.area));
            }
            EleLevel::Floating => Self::to_refs(&self.floating, |e| true),
            EleLevel::Notify => Self::to_refs(&self.notify, |e| true),
        }
    }

    pub(crate) fn get<'a>(&'a self, idx: &EleIndex) -> Option<&'a Element> {
        let eles = match idx.level {
            EleLevel::Base => &self.base,
            EleLevel::Floating => &self.floating,
            EleLevel::Notify => &self.notify,
        };
        return eles.get(idx.idx);
    }
}
