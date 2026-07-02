use bitflags::bitflags;
use ratatui::{
    Frame,
    layout::{Rect, Size},
};

use crate::ui::UIApp;

bitflags! {
    #[derive(Debug, Default)]
     pub struct PreventEventMode: u8 {
        const KEY    = 0b0000_0001;
        const MOUSE   = 0b0000_0010;
    }
}

#[derive(Debug, Default)]
pub(crate) struct ScrollViewRef {
    pub(crate) id: String,
    pub(crate) size: Size,
    pub(crate) area: Rect,
}

#[derive(Debug, Default)]
pub(crate) struct EleOptions {
    pub(crate) prevent_event: PreventEventMode,
    pub(crate) in_scroll_view: Option<ScrollViewRef>,
    pub(crate) input_id: Option<String>,
}

pub(crate) struct EleTempInfo {
    pub(crate) id: String,
    pub(crate) render_fn: Option<Box<dyn FnOnce(&mut Frame, Rect)>>,
    pub(crate) area: Rect,
    pub(crate) opts: Option<EleOptions>,
}

impl UIApp {
    pub(crate) fn eleopts(&self, id: &str) -> Option<EleOptions> {
        return ele_opts_by_id(id);
    }
}

pub(crate) fn ele_opts_by_id(id: &str) -> Option<EleOptions> {
    return None;
}
