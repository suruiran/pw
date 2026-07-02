use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::ui::UIApp;

impl UIApp {
    fn on_key_evt(&mut self, evt: KeyEvent) -> bool {
        if evt.is_release() {
            return true;
        }

        tracing::info!("{:?}", evt);

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
            _ => {
                return true;
            }
        }
    }

    fn on_mouse_ele(&mut self, evt: MouseEvent, id: String) -> bool {
        return true;
    }

    fn on_mouse_evt(&mut self, evt: MouseEvent) -> bool {
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
