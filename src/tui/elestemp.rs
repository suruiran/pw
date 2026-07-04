use std::{cell::RefCell, rc::Rc};

use ratatui::Frame;

use crate::tui::eleinfo::{EleIndex, EleTempInfo};

#[derive(Debug)]
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
pub(crate) struct ElesTemp {
    base: Vec<EleTempInfo>,
    floating: Vec<EleTempInfo>,
    notify: Vec<EleTempInfo>,
}

pub(crate) type ElesTempRef = Rc<RefCell<ElesTemp>>;

impl ElesTemp {
    pub(crate) fn clear(&mut self) {
        self.base.clear();
        self.floating.clear();
        self.notify.clear();
    }

    pub(crate) fn push(&mut self, level: EleLevel, ele: EleTempInfo) {
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

impl ElesTemp {
    fn top_eles(&self) -> &Vec<EleTempInfo> {
        if !self.notify.is_empty() {
            return &self.notify;
        }
        if !self.floating.is_empty() {
            return &self.floating;
        }
        return &self.base;
    }

    pub(crate) fn all_focusable(&self) -> Vec<&EleTempInfo> {
        return self
            .top_eles()
            .iter()
            .filter(|ele| ele.responsive())
            .collect();
    }
}
