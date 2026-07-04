use bitflags::bitflags;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Position, Rect, Size};

use crate::tui::{app::UIApp, eleinfo::Element, layers::EleLevel};

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

impl UIApp {
    fn do_scroll(&self, action: ScrollAction) {
        let mut scrollview = self.scrollview.borrow_mut();
        if scrollview.is_none() {
            return;
        }
        let scrollview = scrollview.as_mut().unwrap();
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
    }

    fn do_change_focus(&self, action: ChangeFocusAction) {}

    fn on_key_evt(&self, evt: KeyEvent, eles: Vec<&Element>) -> bool {
        if evt.is_release() {
            return true;
        }
        match evt.code {
            KeyCode::Esc => {
                return false;
            }
            KeyCode::Char(code) => {
                if evt.modifiers.contains(KeyModifiers::CONTROL) {
                    match code {
                        'c' | 'z' => {
                            return false;
                        }
                        _ => {}
                    }
                }
                return true;
            }
            KeyCode::Down => {
                self.do_change_focus(ChangeFocusAction::Vertical(1));
                return true;
            }
            KeyCode::Up => {
                self.do_change_focus(ChangeFocusAction::Vertical(-1));
                return true;
            }
            KeyCode::Left => {
                self.do_change_focus(ChangeFocusAction::Horizontal(-1));
                return true;
            }
            KeyCode::Right => {
                self.do_change_focus(ChangeFocusAction::Horizontal(1));
                return true;
            }
            KeyCode::Tab => {
                if evt.modifiers.contains(KeyModifiers::SHIFT) {
                    self.do_change_focus(ChangeFocusAction::Index(-1));
                } else {
                    self.do_change_focus(ChangeFocusAction::Index(1));
                }
                return true;
            }
            KeyCode::PageUp => {
                self.do_scroll(ScrollAction::Up | ScrollAction::ByPage);
                return true;
            }
            KeyCode::PageDown => {
                self.do_scroll(ScrollAction::Down | ScrollAction::ByPage);
                return true;
            }
            KeyCode::Home => {
                self.do_scroll(ScrollAction::ToTop);
                return true;
            }
            KeyCode::End => {
                self.do_scroll(ScrollAction::ToBottom);
                return true;
            }
            _ => {
                return true;
            }
        }
    }

    fn find_ele_by_pos(eles: Vec<&Element>, pos: Position) -> Vec<&Element> {
        return eles.into_iter().filter(|e| e.area.contains(pos)).collect();
    }

    fn on_mouse_evt(&self, evt: MouseEvent, eles: Vec<&Element>) -> bool {
        match evt.kind {
            crossterm::event::MouseEventKind::Down(btn) => {
                match btn {
                    crossterm::event::MouseButton::Left => {}
                    _ => {
                        return true;
                    }
                }

                let eles = Self::find_ele_by_pos(
                    eles,
                    Position {
                        x: evt.column,
                        y: evt.row,
                    },
                );
                tracing::info!("CLICK {} {} {:?}", evt.column, evt.row, eles);
            }
            crossterm::event::MouseEventKind::Moved => {}
            crossterm::event::MouseEventKind::ScrollDown => {
                let mut action = ScrollAction::Down;
                if evt.modifiers.contains(KeyModifiers::ALT) {
                    action |= ScrollAction::ByPage;
                }
                self.do_scroll(action);
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                let mut action = ScrollAction::Up;
                if evt.modifiers.contains(KeyModifiers::ALT) {
                    action |= ScrollAction::ByPage;
                }
                self.do_scroll(action);
            }
            _ => {}
        }
        return true;
    }

    pub(crate) fn react(&self, vpsize: Size) -> bool {
        let top_level = { self.layers.borrow().top_level() };
        if top_level == EleLevel::Base {
            let mut layers = self.layers.borrow_mut();
            layers.adjust_base_layer(self.scrollview.clone());
        }
        let layers = self.layers.borrow();
        let eles = layers.all_focusable(vpsize);
        // after this, we can not change `self.layers`.

        if let Some(Event::Key(keyevt)) = self.evt {
            return self.on_key_evt(keyevt, eles);
        }
        if self.mouse_enabled
            && let Some(Event::Mouse(mouseevt)) = self.evt
        {
            return self.on_mouse_evt(mouseevt, eles);
        }
        return true;
    }
}
