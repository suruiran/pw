use std::fmt::Debug;

use ratatui::{
    Frame,
    layout::{Rect, Size},
};

use crate::tui::layers::EleLevel;

#[derive(Debug)]
pub(crate) enum ActiveAction {
    Nothing,
    ShowArguDesc(String),
    ShowCommnadDesc,
    Input,
    AddArgv,
    DelArgv(usize),
}

impl Default for ActiveAction {
    fn default() -> Self {
        return ActiveAction::Nothing;
    }
}

#[derive(Debug, Default)]
pub(crate) struct EleOptions {
    pub(crate) in_scroll_view: bool,
    pub(crate) input_id: Option<String>,
    pub(crate) on_active: ActiveAction,
}

impl EleOptions {
    pub(crate) fn new(in_scrollview: bool) -> Self {
        let mut ele: Self = Default::default();
        ele.in_scroll_view = in_scrollview;
        return ele;
    }

    pub(crate) fn set_input_id(self, id: &str) -> Self {
        let mut ele = self;
        ele.input_id = Some(id.to_string());
        return ele;
    }

    pub(crate) fn set_action(self, action: ActiveAction) -> Self {
        let mut ele = self;
        ele.on_active = action;
        return ele;
    }
}

#[derive(Debug, Default)]
pub(crate) struct EleIndex {
    pub(crate) level: EleLevel,
    pub(crate) idx: usize,
}

#[derive(Default)]
pub(crate) struct Element {
    pub(crate) index: EleIndex,
    pub(crate) id: String,
    pub(crate) render_fn: Option<Box<dyn FnOnce(&mut Frame, Rect)>>,
    pub(crate) area: Rect,
    pub(crate) opts: Option<EleOptions>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EleTempInfo")
            .field("index", &self.index)
            .field("id", &self.id)
            .field("area", &self.area)
            .field("opts", &self.opts)
            .finish()
    }
}

impl Element {
    pub(crate) fn responsive(&self) -> bool {
        if let Some(opts) = self.opts.as_ref() {
            match opts.on_active {
                ActiveAction::Nothing => {
                    return false;
                }
                _ => {
                    return true;
                }
            }
        }
        return false;
    }
}
