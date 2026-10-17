//! In-process inline prompts. No runtime, Ratatui, product writes or process exits.
#![forbid(unsafe_code)]

mod outcome;
mod theme;
pub use outcome::{PromptError, PromptOutcome};
pub use theme::{PromptTheme, visible_text};

use std::{
    io::{self, IsTerminal},
    marker::PhantomData,
    sync::{Mutex, MutexGuard},
};
use terminal_ui_contract::theme::ColorLevel;

static PRESENTATION: Mutex<()> = Mutex::new(());

/// Stable product-owned identity; disabled choices cannot be returned.
#[derive(Clone, Debug)]
pub struct Choice<T> {
    pub id: T,
    pub label: String,
    pub description: String,
    pub disabled: bool,
}
impl<T> Choice<T> {
    pub fn new(id: T, label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            description: description.into(),
            disabled: false,
        }
    }
}

/// Owns cliclack's global theme until dropped. Never acquire on JSON/help paths.
/// A second session fails rather than deadlocking. After the session, the default
/// cliclack theme and previous console color policy are restored. Existing callers
/// with custom cliclack themes should configure their theme again after this owner.
pub struct PromptSession {
    _owner: MutexGuard<'static, ()>,
    previous_color: bool,
    previous_stdout_color: bool,
    theme: PromptTheme,
}

impl PromptSession {
    pub fn new(mut theme: PromptTheme) -> Result<Self, PromptError> {
        if !interactive_capable(
            io::stdin().is_terminal(),
            io::stderr().is_terminal(),
            std::env::var("TERM").ok().as_deref(),
        ) {
            return Err(PromptError::NotTerminal);
        }
        let owner = PRESENTATION
            .try_lock()
            .map_err(|_| PromptError::TerminalBusy)?;
        if std::env::var_os("NO_COLOR").is_some() {
            theme.color_level = ColorLevel::None;
        }
        let previous_color = console::colors_enabled_stderr();
        let previous_stdout_color = console::colors_enabled();
        console::set_colors_enabled(theme.color_level != ColorLevel::None);
        console::set_colors_enabled_stderr(theme.color_level != ColorLevel::None);
        cliclack::set_theme(theme::InlineTheme(theme));
        Ok(Self {
            _owner: owner,
            previous_color,
            previous_stdout_color,
            theme,
        })
    }

    fn label(&self, text: &str) -> String {
        visible_text(
            text,
            usize::from(console::Term::stderr().size().1).saturating_sub(6),
        )
    }
    fn rows(&self) -> usize {
        usize::from(console::Term::stderr().size().0)
            .saturating_sub(5)
            .clamp(1, 8)
    }

    pub fn select<T: Clone + Eq>(
        &mut self,
        prompt: &str,
        choices: &[Choice<T>],
        default: &T,
    ) -> PromptOutcome<T> {
        if !valid_choices(choices, std::slice::from_ref(default)) {
            return PromptOutcome::Error(PromptError::InvalidChoices);
        }
        let mut select = cliclack::select(self.label(prompt))
            .initial_value(default.clone())
            .max_rows(self.rows());
        for c in choices.iter().filter(|c| !c.disabled) {
            select = select.item(
                c.id.clone(),
                self.label(&c.label),
                self.label(&c.description),
            );
        }
        select.interact().into()
    }

    pub fn multiselect<T: Clone + Eq>(
        &mut self,
        prompt: &str,
        choices: &[Choice<T>],
        defaults: &[T],
        required: bool,
    ) -> PromptOutcome<Vec<T>> {
        if !valid_choices(choices, defaults) {
            return PromptOutcome::Error(PromptError::InvalidChoices);
        }
        let mut select = cliclack::multiselect(self.label(prompt))
            .initial_values(defaults.to_vec())
            .required(required)
            .max_rows(self.rows());
        for c in choices.iter().filter(|c| !c.disabled) {
            select = select.item(
                c.id.clone(),
                self.label(&c.label),
                self.label(&c.description),
            );
        }
        select.interact().into()
    }

    pub fn input(&mut self, prompt: &str, default: &str, required: bool) -> PromptOutcome<String> {
        cliclack::input(self.label(prompt))
            .default_input(default)
            .required(required)
            .interact()
            .into()
    }
    pub fn confirm(&mut self, prompt: &str, default: bool) -> PromptOutcome<bool> {
        cliclack::confirm(self.label(prompt))
            .initial_value(default)
            .interact()
            .into()
    }
    pub fn progress(&mut self, message: &str, length: u64) -> PromptProgress<'_> {
        let bar = cliclack::progress_bar(length);
        bar.start(self.label(message));
        PromptProgress {
            bar,
            cancelled: self.theme.cancelled(),
            finished: false,
            _owner: PhantomData,
        }
    }
}
impl Drop for PromptSession {
    fn drop(&mut self) {
        let _ = console::Term::stderr().show_cursor();
        cliclack::reset_theme();
        console::set_colors_enabled_stderr(self.previous_color);
        console::set_colors_enabled(self.previous_stdout_color);
    }
}

pub struct PromptProgress<'a> {
    bar: cliclack::ProgressBar,
    cancelled: &'static str,
    finished: bool,
    _owner: PhantomData<&'a mut PromptSession>,
}
impl PromptProgress<'_> {
    pub fn advance(&self, amount: u64) {
        self.bar.inc(amount);
    }
    pub fn finish(mut self, message: &str) {
        self.bar.stop(visible_text(
            message,
            usize::from(console::Term::stderr().size().1).saturating_sub(6),
        ));
        self.finished = true;
    }
}
impl Drop for PromptProgress<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.bar.cancel(self.cancelled);
        }
    }
}

pub fn interactive_capable(stdin: bool, stderr: bool, term: Option<&str>) -> bool {
    stdin && stderr && term != Some("dumb")
}

fn valid_choices<T: Eq>(choices: &[Choice<T>], defaults: &[T]) -> bool {
    choices.iter().any(|c| !c.disabled)
        && choices
            .iter()
            .enumerate()
            .all(|(i, c)| !choices[..i].iter().any(|p| p.id == c.id))
        && defaults
            .iter()
            .all(|d| choices.iter().any(|c| !c.disabled && &c.id == d))
        && defaults
            .iter()
            .enumerate()
            .all(|(i, d)| !defaults[..i].contains(d))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_disabled_defaults_never_submit() {
        let mut choices = vec![Choice::new("a", "甲", ""), Choice::new("b", "乙", "")];
        choices[1].disabled = true;
        assert!(valid_choices(&choices, &["a"]));
        assert!(!valid_choices(&choices, &["b"]));
        assert!(!valid_choices(&choices, &["missing"]));
        assert!(!valid_choices(&choices, &["a", "a"]));
        choices[1].id = "a";
        assert!(!valid_choices(&choices, &[]));
        assert!(!valid_choices::<u8>(&[], &[]));
    }
    #[test]
    fn cancellation_is_distinct_from_io_error() {
        assert!(matches!(
            PromptOutcome::<()>::from(Err(io::ErrorKind::Interrupted.into())),
            PromptOutcome::Cancelled
        ));
        assert!(matches!(
            PromptOutcome::<()>::from(Err(io::ErrorKind::BrokenPipe.into())),
            PromptOutcome::Error(_)
        ));
    }
    #[test]
    fn tty_and_display_safety() {
        if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            assert!(matches!(
                PromptSession::new(PromptTheme::default()),
                Err(PromptError::NotTerminal)
            ));
        }
        assert!(!interactive_capable(false, true, None));
        assert!(!interactive_capable(true, false, None));
        assert!(!interactive_capable(true, true, Some("dumb")));
        assert!(interactive_capable(true, true, None));
        for width in [1, 2, 50, 70, 100] {
            let text = visible_text(&"中文e\u{301}🎉".repeat(40), width);
            assert!(terminal_ui_contract::text::display_width(&text) <= width);
        }
        assert!(!visible_text("x\x1b[31m\ny", 80).contains('\x1b'));
    }
}
