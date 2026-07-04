use std::{cell::RefCell, rc::Rc};

use ratatui::{
    Frame,
    layout::{Rect, Size},
};

use crate::tui::{
    app::ScrollViewInfoRef,
    eleinfo::{EleIndex, Element},
};

#[derive(Debug, PartialEq)]
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
}

pub(crate) type UILayersRef = Rc<RefCell<UILayers>>;

impl UILayers {
    pub(crate) fn clear(&mut self) {
        self.base.clear();
        self.floating.clear();
        self.notify.clear();
    }

    pub(crate) fn push(&mut self, level: EleLevel, ele: Element) {
        let eles = match level {
            EleLevel::Base => &mut self.base,
            EleLevel::Floating => &mut self.floating,
            EleLevel::Notify => &mut self.notify,
        };

        let mut ele = ele;
        ele.index.level = level;
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
        let offset = scrollview.borrow().as_ref().unwrap().state.offset();
        self.base.iter_mut().for_each(|ele| {
            ele.area.x += offset.x;
            ele.area.y += offset.y;

            tracing::info!("{} {:?} {}", &ele.id, ele.area, offset);
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
}
