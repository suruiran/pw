use bitflags::bitflags;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::tui::ui::UIApp;

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
    fn do_scroll(&mut self, action: ScrollAction) {
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

    fn do_change_focus(&mut self, action: ChangeFocusAction) {}

    fn on_key_evt(&mut self, evt: KeyEvent) -> bool {
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

    fn on_mouse_ele(&mut self, evt: MouseEvent, id: String) -> bool {
        return true;
    }

    fn on_mouse_evt(&mut self, evt: MouseEvent) -> bool {
        match evt.kind {
            crossterm::event::MouseEventKind::Down(mouse_button) => {}
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

    pub(crate) fn react(&mut self) -> bool {
        if let Some(Event::Key(keyevt)) = self.evt {
            return self.on_key_evt(keyevt);
        }
        if self.mouse_enabled
            && let Some(Event::Mouse(mouseevt)) = self.evt
        {
            return self.on_mouse_evt(mouseevt);
        }
        return true;
    }
}
