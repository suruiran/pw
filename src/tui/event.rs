use bitflags::bitflags;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Position, Size};

use crate::tui::{app::TUIApp, element::Element};

bitflags! {
    #[derive(Debug, Default)]
    pub struct ScrollAction: u8{
        const Up = 0b0000_0001;
        const Down = 0b0000_0010;
        const ByPage = 0b0000_0100;

        const ToTop = 0b0000_1000;
        const ToBottom = 0b0001_0000;
    }
}

pub(crate) enum ChangeFocusAction {
    Horizontal(i8),
    Vertical(i8),
    Index(i8),
}

#[derive(Debug, PartialEq)]
pub(crate) enum EvtReturn {
    Ignore,
    Ok,
    Exit,
}

impl TUIApp {
    fn do_scroll(&mut self, action: ScrollAction) {
        let ctx = &mut self.ctx;
        ctx.with_scrollview_mut(|sv| {
            if sv.is_none() {
                return;
            }
            let scrollview = sv.as_mut().unwrap();
            let scrollview_state = &mut scrollview.state;

            if action.contains(ScrollAction::ToTop) {
                scrollview_state.scroll_to_top();
                return;
            }

            if action.contains(ScrollAction::ToBottom) {
                scrollview_state.scroll_to_bottom();
                return;
            }

            if action.contains(ScrollAction::Up) {
                if action.contains(ScrollAction::ByPage) {
                    scrollview_state.scroll_page_up();
                    return;
                }
                scrollview_state.scroll_up();
                return;
            }
            if action.contains(ScrollAction::ByPage) {
                scrollview_state.scroll_page_down();
                return;
            }
            scrollview_state.scroll_down();
        });
    }

    fn do_change_focus(&self, action: ChangeFocusAction) {}

    fn on_key_evt(&mut self, evt: KeyEvent, vpsize: Size) -> EvtReturn {
        if evt.is_release() {
            return EvtReturn::Ignore;
        }
        match evt.code {
            KeyCode::Esc => {
                return EvtReturn::Exit;
            }
            KeyCode::Char(code) => {
                if evt.modifiers.contains(KeyModifiers::CONTROL) {
                    match code {
                        'c' | 'z' => {
                            return EvtReturn::Exit;
                        }
                        _ => {}
                    }
                }
                return EvtReturn::Ignore;
            }
            KeyCode::Down => {
                self.do_change_focus(ChangeFocusAction::Vertical(1));
                return EvtReturn::Ok;
            }
            KeyCode::Up => {
                self.do_change_focus(ChangeFocusAction::Vertical(-1));
                return EvtReturn::Ok;
            }
            KeyCode::Left => {
                self.do_change_focus(ChangeFocusAction::Horizontal(-1));
                return EvtReturn::Ok;
            }
            KeyCode::Right => {
                self.do_change_focus(ChangeFocusAction::Horizontal(1));
                return EvtReturn::Ok;
            }
            KeyCode::Tab => {
                if evt.modifiers.contains(KeyModifiers::SHIFT) {
                    self.do_change_focus(ChangeFocusAction::Index(-1));
                } else {
                    self.do_change_focus(ChangeFocusAction::Index(1));
                }
                return EvtReturn::Ok;
            }
            KeyCode::PageUp => {
                self.do_scroll(ScrollAction::Up | ScrollAction::ByPage);
                return EvtReturn::Ok;
            }
            KeyCode::PageDown => {
                self.do_scroll(ScrollAction::Down | ScrollAction::ByPage);
                return EvtReturn::Ok;
            }
            KeyCode::Home => {
                self.do_scroll(ScrollAction::ToTop);
                return EvtReturn::Ok;
            }
            KeyCode::End => {
                self.do_scroll(ScrollAction::ToBottom);
                return EvtReturn::Ok;
            }
            _ => {
                return EvtReturn::Ignore;
            }
        }
    }

    fn find_ele_by_pos(eles: Vec<&Element>, pos: Position) -> Vec<&Element> {
        return eles.into_iter().filter(|e| e.area.contains(pos)).collect();
    }

    pub(crate) fn with_focusable<R>(&self, vpsize: Size, f: impl FnOnce(Vec<&Element>) -> R) -> R {
        return self.ctx.with_focusables(vpsize, f);
    }

    fn on_mouse_evt(&mut self, evt: MouseEvent, vpsize: Size) -> EvtReturn {
        match evt.kind {
            crossterm::event::MouseEventKind::Down(btn) => {
                match btn {
                    crossterm::event::MouseButton::Left => {}
                    _ => {
                        return EvtReturn::Ignore;
                    }
                }
                return self.with_focusable(vpsize, |eles| {
                    let eles = Self::find_ele_by_pos(
                        eles,
                        Position {
                            x: evt.column,
                            y: evt.row,
                        },
                    );
                    tracing::info!(
                        "CLICK {} {} {:?}",
                        evt.column,
                        evt.row,
                        eles.iter().map(|v| v.id.clone()).collect::<Vec<_>>(),
                    );
                    return EvtReturn::Ok;
                });
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                let mut action = ScrollAction::Down;
                if evt.modifiers.contains(KeyModifiers::ALT) {
                    action |= ScrollAction::ByPage;
                }
                self.do_scroll(action);
                return EvtReturn::Ok;
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                let mut action = ScrollAction::Up;
                if evt.modifiers.contains(KeyModifiers::ALT) {
                    action |= ScrollAction::ByPage;
                }
                self.do_scroll(action);
                return EvtReturn::Ok;
            }
            _ => return EvtReturn::Ignore,
        }
    }

    pub(crate) fn react(&mut self, evt: Event, vpsize: Size) -> EvtReturn {
        match evt {
            Event::Mouse(evt) => {
                return self.on_mouse_evt(evt, vpsize);
            }
            Event::Key(evt) => {
                return self.on_key_evt(evt, vpsize);
            }
            Event::Resize(_, _) => {
                return EvtReturn::Ok;
            }
            _ => {
                return EvtReturn::Ignore;
            }
        }
    }
}
