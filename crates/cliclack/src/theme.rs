use console::Style;
use terminal_ui_contract::{locale::Locale, text::truncate_to_width, theme::ColorLevel};

/// Session-scoped presentation only. Products continue to own translations of domain text.
#[derive(Clone, Copy, Debug)]
pub struct PromptTheme {
    pub locale: Locale,
    pub color_level: ColorLevel,
}

impl Default for PromptTheme {
    fn default() -> Self {
        Self {
            locale: Locale::EnUs,
            color_level: ColorLevel::Ansi16,
        }
    }
}

impl PromptTheme {
    pub(crate) fn cancelled(self) -> &'static str {
        match self.locale {
            Locale::EnUs => "Cancelled",
            Locale::ZhCn => "已取消",
        }
    }
}

pub(crate) struct InlineTheme(pub PromptTheme);
impl cliclack::Theme for InlineTheme {
    fn state_symbol_color(&self, state: &cliclack::ThemeState) -> Style {
        if self.0.color_level == ColorLevel::None {
            return Style::new();
        }
        match state {
            cliclack::ThemeState::Cancel | cliclack::ThemeState::Error(_) => Style::new().red(),
            _ => Style::new().cyan(),
        }
    }
    fn format_footer(&self, state: &cliclack::ThemeState) -> String {
        if matches!(state, cliclack::ThemeState::Cancel) {
            format!("└  {}\n", self.0.cancelled())
        } else {
            String::new()
        }
    }
}

/// Remove terminal controls and truncate at grapheme/display-cell boundaries.
pub fn visible_text(value: &str, columns: usize) -> String {
    let safe: String = value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    truncate_to_width(&safe, columns.max(1))
}
