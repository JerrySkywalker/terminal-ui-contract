//! Shared overlay and staged-settings interaction state.

/// The generic shared overlay currently required by both consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayState {
    None,
    Help,
}

impl OverlayState {
    #[must_use]
    pub const fn is_open(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Opens Help, or leaves an existing modal owner untouched.
    pub fn open_help(&mut self) {
        if !self.is_open() {
            *self = Self::Help;
        }
    }

    /// Dismisses Help for any of its shared dismissal keys.
    #[must_use]
    pub fn dismiss_with(&mut self, key: OverlayDismissKey) -> bool {
        if self.is_open()
            && matches!(
                key,
                OverlayDismissKey::Escape | OverlayDismissKey::Help | OverlayDismissKey::Quit
            )
        {
            *self = Self::None;
            true
        } else {
            false
        }
    }
}

/// The only keys that dismiss a generic Help overlay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayDismissKey {
    Escape,
    Help,
    Quit,
}

/// Result of a quit request against an in-memory staged draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuitDisposition {
    Quit,
    ConfirmDiscard,
}

/// Explicit answer to a pending discard confirmation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscardDecision {
    Cancel,
    Discard,
}

/// Shared settings interaction state. Values remain owned by the consumer;
/// this type owns only the state transitions that prevent UX drift.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingsEditor<Field> {
    selected_field: Field,
    editing: bool,
    discard_confirmation: bool,
}

impl<Field: Copy + Eq> SettingsEditor<Field> {
    #[must_use]
    pub const fn new(selected_field: Field) -> Self {
        Self {
            selected_field,
            editing: false,
            discard_confirmation: false,
        }
    }

    #[must_use]
    pub const fn selected_field(self) -> Field {
        self.selected_field
    }

    #[must_use]
    pub const fn is_editing(self) -> bool {
        self.editing
    }

    #[must_use]
    pub const fn awaiting_discard_confirmation(self) -> bool {
        self.discard_confirmation
    }

    /// Implements Enter as the explicit edit/done toggle.
    pub fn enter_or_finish(&mut self) {
        self.editing = !self.editing;
    }

    /// Moves the selected setting only while editing. `ordered` is owned by
    /// the consumer's domain, while wrapping behavior is shared.
    #[must_use]
    pub fn move_field(&mut self, ordered: &[Field], delta: isize) -> bool {
        if !self.editing || ordered.is_empty() {
            return false;
        }
        let Some(index) = ordered
            .iter()
            .position(|field| *field == self.selected_field)
        else {
            return false;
        };
        let next = (index as isize + delta).rem_euclid(ordered.len() as isize) as usize;
        self.selected_field = ordered[next];
        true
    }

    /// Requires confirmation only when a staged change exists.
    pub fn request_quit(&mut self, dirty: bool) -> QuitDisposition {
        if dirty {
            self.discard_confirmation = true;
            QuitDisposition::ConfirmDiscard
        } else {
            QuitDisposition::Quit
        }
    }

    /// Resolves the pending confirmation without silently discarding a draft.
    #[must_use]
    pub fn resolve_discard(&mut self, decision: DiscardDecision) -> bool {
        if !self.discard_confirmation {
            return false;
        }
        self.discard_confirmation = false;
        matches!(decision, DiscardDecision::Discard)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiscardDecision, OverlayDismissKey, OverlayState, QuitDisposition, SettingsEditor,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Field {
        Language,
        Color,
    }

    #[test]
    fn help_owns_dismissal_until_explicitly_closed() {
        let mut overlay = OverlayState::None;
        overlay.open_help();
        assert!(overlay.is_open());
        assert!(overlay.dismiss_with(OverlayDismissKey::Help));
        assert!(!overlay.is_open());
    }

    #[test]
    fn settings_field_motion_is_edit_scoped_and_dirty_quit_is_confirmed() {
        let mut editor = SettingsEditor::new(Field::Language);
        assert!(!editor.move_field(&[Field::Language, Field::Color], 1));
        editor.enter_or_finish();
        assert!(editor.move_field(&[Field::Language, Field::Color], 1));
        assert_eq!(editor.selected_field(), Field::Color);
        assert_eq!(editor.request_quit(true), QuitDisposition::ConfirmDiscard);
        assert!(editor.awaiting_discard_confirmation());
        assert!(!editor.resolve_discard(DiscardDecision::Cancel));
        assert!(!editor.awaiting_discard_confirmation());
        assert_eq!(editor.request_quit(true), QuitDisposition::ConfirmDiscard);
        assert!(editor.resolve_discard(DiscardDecision::Discard));
    }
}
