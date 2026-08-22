//! Normalized input admission policy.

/// A terminal-library-neutral key-event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputKind {
    Press,
    Repeat,
    Release,
}

/// Enforces the shared press-only Human interaction policy.
#[must_use]
pub const fn admits_input(kind: InputKind) -> bool {
    matches!(kind, InputKind::Press)
}

#[cfg(test)]
mod tests {
    use super::{InputKind, admits_input};

    #[test]
    fn repeat_and_release_are_never_actions() {
        assert!(admits_input(InputKind::Press));
        assert!(!admits_input(InputKind::Repeat));
        assert!(!admits_input(InputKind::Release));
    }
}
