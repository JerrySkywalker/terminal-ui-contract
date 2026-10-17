use std::{fmt, io};

/// A choice is a draft intent; submitting it never executes a product action.
#[derive(Debug)]
pub enum PromptOutcome<T> {
    Submitted(T),
    Cancelled,
    Error(PromptError),
}

#[derive(Debug)]
pub enum PromptError {
    NotTerminal,
    InvalidChoices,
    TerminalBusy,
    Io(io::Error),
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTerminal => f.write_str("interactive input requires a terminal"),
            Self::InvalidChoices => f.write_str("invalid, duplicate, disabled, or empty choices"),
            Self::TerminalBusy => f.write_str("another prompt session owns terminal presentation"),
            Self::Io(_) => f.write_str("terminal input/output failed"),
        }
    }
}
impl std::error::Error for PromptError {}

impl<T> From<io::Result<T>> for PromptOutcome<T> {
    fn from(value: io::Result<T>) -> Self {
        match value {
            Ok(value) => Self::Submitted(value),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Self::Cancelled,
            Err(error) => Self::Error(PromptError::Io(error)),
        }
    }
}
