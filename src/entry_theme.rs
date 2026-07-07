use ratatui::style::Style;

//TODO fill EntryTheme

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct EntryTheme {}

pub(crate) type EntryThemeRef = std::rc::Rc<EntryTheme>;

impl EntryTheme {
    pub(crate) fn argu_label_sep_token(&self) -> &'static str {
        return " / ";
    }

    pub(crate) fn argu_label_sep_style(&self) -> Style {
        return Style::new().fg(ratatui::style::Color::Rgb(0, 0, 0));
    }

    pub(crate) fn argu_label_name_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_label_type_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_label_required_token(&self) -> &'static str {
        return "required";
    }

    pub(crate) fn argu_label_required_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_label_desc_indicator_token(&self) -> &'static str {
        return "[?]";
    }

    pub(crate) fn argu_label_desc_indicator_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_label_desc_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_label_add_value_token(&self) -> &'static str {
        return "[+]";
    }

    pub(crate) fn argu_label_add_value_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_wrapper_border_style(&self) -> Style {
        return Style::new();
    }

    pub(crate) fn argu_input_border_style(&self, focused: bool) -> Style {
        if focused {
            return Style::new().fg(ratatui::style::Color::Green);
        }
        return Style::new();
    }

    pub(crate) fn argu_input_style(&self, focused: bool) -> Style {
        if focused {
            return Style::new().fg(ratatui::style::Color::Blue);
        }
        return Style::new();
    }
}
