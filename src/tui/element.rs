use std::fmt::Debug;

use ratatui::{
    layout::Rect,
    widgets::{StatefulWidget, Widget},
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
    pub(crate) auto_focusable: bool,
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

    pub(crate) fn auto_focusable(self) -> Self {
        let mut ele = self;
        ele.auto_focusable = true;
        return ele;
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct EleIndex {
    pub(crate) level: EleLevel,
    pub(crate) idx: usize,
}

#[derive(Default)]
pub(crate) struct Element {
    pub(crate) index: EleIndex,
    pub(crate) id: String,
    pub(crate) buf: Option<ratatui::buffer::Buffer>,
    pub(crate) area: Rect,
    pub(crate) opts: Option<EleOptions>,
}

impl Element {
    pub(crate) fn new(level: EleLevel, id: &str, area: Rect, opts: Option<EleOptions>) -> Self {
        let mut ele = Self {
            index: Default::default(),
            id: id.to_string(),
            buf: None,
            area,
            opts,
        };
        ele.index.level = level;
        return ele;
    }

    pub(crate) fn plain<W: Widget>(self, w: W, clear: bool) -> Self {
        let mut buf = ratatui::buffer::Buffer::empty(self.area);
        if clear {
            ratatui::widgets::Clear.render(self.area, &mut buf);
        }
        w.render(self.area, &mut buf);

        let mut ele = self;
        ele.buf = Some(buf);
        return ele;
    }

    pub(crate) fn stateful<W: StatefulWidget>(self, w: W, s: &mut W::State, clear: bool) -> Self
    where
        <W as StatefulWidget>::State: Sized,
    {
        let mut buf = ratatui::buffer::Buffer::empty(self.area);
        if clear {
            ratatui::widgets::Clear.render(self.area, &mut buf);
        }
        w.render(self.area, &mut buf, s);

        let mut ele = self;
        ele.buf = Some(buf);
        return ele;
    }
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
