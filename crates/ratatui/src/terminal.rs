//! Paired owned terminal lifecycle. Tests use an injected terminal, never the daily terminal.
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{
    fmt,
    io::{self, Write},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalStep {
    Raw,
    Alternate,
    Cursor,
    Mouse,
    Paste,
}
pub trait TerminalOps {
    fn apply(&mut self, step: TerminalStep, enter: bool) -> io::Result<()>;
}

#[derive(Debug)]
pub struct LifecycleError {
    pub failures: Vec<(TerminalStep, String)>,
}
impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "terminal lifecycle failed: {:?}", self.failures)
    }
}
impl std::error::Error for LifecycleError {}

pub struct TerminalGuard<O: TerminalOps> {
    ops: O,
    active: Vec<TerminalStep>,
}
impl<O: TerminalOps> TerminalGuard<O> {
    pub fn enter(ops: O, mouse: bool) -> Result<Self, LifecycleError> {
        let mut guard = Self {
            ops,
            active: vec![],
        };
        for step in [
            TerminalStep::Raw,
            TerminalStep::Alternate,
            TerminalStep::Cursor,
            TerminalStep::Mouse,
            TerminalStep::Paste,
        ] {
            if step == TerminalStep::Mouse && !mouse {
                continue;
            }
            // Record before entering: an I/O failure can follow a partial side effect.
            guard.active.push(step);
            if let Err(error) = guard.ops.apply(step, true) {
                let mut failures = vec![(step, error.to_string())];
                if let Err(cleanup) = guard.finish() {
                    failures.extend(cleanup.failures);
                }
                return Err(LifecycleError { failures });
            }
        }
        Ok(guard)
    }
    /// Explicit finish returns cleanup failures. Drop retries outstanding steps.
    pub fn finish(&mut self) -> Result<(), LifecycleError> {
        let mut failures = vec![];
        let mut retry = vec![];
        for step in self.active.drain(..).rev() {
            if let Err(error) = self.ops.apply(step, false) {
                failures.push((step, error.to_string()));
                retry.push(step);
            }
        }
        retry.reverse();
        self.active = retry;
        if failures.is_empty() {
            Ok(())
        } else {
            Err(LifecycleError { failures })
        }
    }
}
impl<O: TerminalOps> Drop for TerminalGuard<O> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

pub struct CrosstermOps<W: Write>(pub W);
impl<W: Write> TerminalOps for CrosstermOps<W> {
    fn apply(&mut self, step: TerminalStep, enter: bool) -> io::Result<()> {
        match (step, enter) {
            (TerminalStep::Raw, true) => enable_raw_mode(),
            (TerminalStep::Raw, false) => disable_raw_mode(),
            (TerminalStep::Alternate, true) => execute!(self.0, EnterAlternateScreen),
            (TerminalStep::Alternate, false) => execute!(self.0, LeaveAlternateScreen),
            (TerminalStep::Cursor, true) => execute!(self.0, Hide),
            (TerminalStep::Cursor, false) => execute!(self.0, Show),
            (TerminalStep::Mouse, true) => execute!(self.0, EnableMouseCapture),
            (TerminalStep::Mouse, false) => execute!(self.0, DisableMouseCapture),
            (TerminalStep::Paste, true) => execute!(self.0, EnableBracketedPaste),
            (TerminalStep::Paste, false) => execute!(self.0, DisableBracketedPaste),
        }
    }
}
