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

/// Modifiers are retained even when a terminal cannot produce a particular chord.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1);
    pub const CONTROL: Self = Self(2);
    pub const ALT: Self = Self(4);
    pub const SUPER: Self = Self(8);
    pub const HYPER: Self = Self(16);
    pub const META: Self = Self(32);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn is_text(self) -> bool {
        self.0 & !Self::SHIFT.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Insert,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Function(u8),
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyChord {
    pub const fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }
    /// Character case is significant. SHIFT on an already-shifted character
    /// is canonicalized; CONTROL/ALT/SUPER/HYPER/META are never discarded.
    pub fn normalized(self) -> Self {
        match self.code {
            KeyCode::Char(c)
                if self.modifiers.contains(Modifiers::SHIFT) && !c.is_ascii_digit() =>
            {
                Self {
                    code: KeyCode::Char(if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }),
                    modifiers: Modifiers(self.modifiers.0 & !Modifiers::SHIFT.0),
                }
            }
            KeyCode::BackTab => Self::new(
                KeyCode::BackTab,
                Modifiers(self.modifiers.0 & !Modifiers::SHIFT.0),
            ),
            KeyCode::Tab if self.modifiers.contains(Modifiers::SHIFT) => Self::new(
                KeyCode::BackTab,
                Modifiers(self.modifiers.0 & !Modifiers::SHIFT.0),
            ),
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub chord: KeyChord,
    pub kind: InputKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseKind {
    Press(MouseButton),
    Release(MouseButton),
    Drag(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseEvent {
    pub column: u16,
    pub row: u16,
    pub kind: MouseKind,
    pub modifiers: Modifiers,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Key(KeyEvent),
    Paste(String),
    Mouse(MouseEvent),
    Resize { columns: u16, rows: u16 },
    FocusGained,
    FocusLost,
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
