//! Contextual footer grammar.

/// The active Human interaction state for the shared footer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FooterState {
    NormalNavigation,
    LocalList,
    Detail,
    SettingsNormal,
    SettingsEdit,
    DirtyDraft,
    DiscardConfirmation,
    HelpOverlay,
    ConflictWarning,
}

/// A localized Human action supplied by the consumer's catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FooterAction {
    Navigate,
    Open,
    Help,
    Refresh,
    Quit,
    Select,
    Back,
    Page,
    Search,
    Filter,
    Sort,
    Edit,
    Change,
    Apply,
    Revert,
    Cancel,
    Discard,
    Dismiss,
}

/// One key/action clause in the Human sentence grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FooterHint {
    pub key: &'static str,
    pub action: FooterAction,
}

const NORMAL_NAVIGATION: &[FooterHint] = &[
    FooterHint {
        key: "↑↓",
        action: FooterAction::Navigate,
    },
    FooterHint {
        key: "Enter",
        action: FooterAction::Open,
    },
    FooterHint {
        key: "?",
        action: FooterAction::Help,
    },
    FooterHint {
        key: "r",
        action: FooterAction::Refresh,
    },
    FooterHint {
        key: "q",
        action: FooterAction::Quit,
    },
];
const LOCAL_LIST: &[FooterHint] = &[
    FooterHint {
        key: "↑↓",
        action: FooterAction::Select,
    },
    FooterHint {
        key: "Enter",
        action: FooterAction::Open,
    },
    FooterHint {
        key: "/",
        action: FooterAction::Search,
    },
    FooterHint {
        key: "f",
        action: FooterAction::Filter,
    },
    FooterHint {
        key: "s",
        action: FooterAction::Sort,
    },
];
const DETAIL: &[FooterHint] = &[
    FooterHint {
        key: "PgUp/PgDn",
        action: FooterAction::Page,
    },
    FooterHint {
        key: "Esc",
        action: FooterAction::Back,
    },
    FooterHint {
        key: "?",
        action: FooterAction::Help,
    },
    FooterHint {
        key: "r",
        action: FooterAction::Refresh,
    },
];
const SETTINGS_NORMAL: &[FooterHint] = &[
    FooterHint {
        key: "Enter",
        action: FooterAction::Edit,
    },
    FooterHint {
        key: "?",
        action: FooterAction::Help,
    },
    FooterHint {
        key: "q",
        action: FooterAction::Quit,
    },
];
const SETTINGS_EDIT: &[FooterHint] = &[
    FooterHint {
        key: "↑↓",
        action: FooterAction::Select,
    },
    FooterHint {
        key: "←→",
        action: FooterAction::Change,
    },
    FooterHint {
        key: "Enter",
        action: FooterAction::Edit,
    },
    FooterHint {
        key: "a",
        action: FooterAction::Apply,
    },
    FooterHint {
        key: "r",
        action: FooterAction::Revert,
    },
];
const DIRTY_DRAFT: &[FooterHint] = &[
    FooterHint {
        key: "a",
        action: FooterAction::Apply,
    },
    FooterHint {
        key: "r",
        action: FooterAction::Revert,
    },
    FooterHint {
        key: "q",
        action: FooterAction::Quit,
    },
];
const DISCARD_CONFIRMATION: &[FooterHint] = &[
    FooterHint {
        key: "Esc",
        action: FooterAction::Cancel,
    },
    FooterHint {
        key: "Enter",
        action: FooterAction::Discard,
    },
];
const HELP_OVERLAY: &[FooterHint] = &[FooterHint {
    key: "Esc/?/q",
    action: FooterAction::Dismiss,
}];
const CONFLICT_WARNING: &[FooterHint] = &[
    FooterHint {
        key: "r",
        action: FooterAction::Revert,
    },
    FooterHint {
        key: "?",
        action: FooterAction::Help,
    },
    FooterHint {
        key: "q",
        action: FooterAction::Quit,
    },
];

impl FooterState {
    /// The small, state-owned set of actions that belongs in the footer.
    #[must_use]
    pub const fn hints(self) -> &'static [FooterHint] {
        match self {
            Self::NormalNavigation => NORMAL_NAVIGATION,
            Self::LocalList => LOCAL_LIST,
            Self::Detail => DETAIL,
            Self::SettingsNormal => SETTINGS_NORMAL,
            Self::SettingsEdit => SETTINGS_EDIT,
            Self::DirtyDraft => DIRTY_DRAFT,
            Self::DiscardConfirmation => DISCARD_CONFIRMATION,
            Self::HelpOverlay => HELP_OVERLAY,
            Self::ConflictWarning => CONFLICT_WARNING,
        }
    }
}

/// Formats the shared Human footer grammar with exactly two spaces between
/// clauses. The consumer supplies locale-specific action wording.
#[must_use]
pub fn format_footer(
    state: FooterState,
    action_text: impl Fn(FooterAction) -> &'static str,
) -> String {
    state
        .hints()
        .iter()
        .map(|hint| format!("{} {}", hint.key, action_text(hint.action)))
        .collect::<Vec<_>>()
        .join("  ")
}

#[cfg(test)]
mod tests {
    use super::{FooterAction, FooterState, format_footer};

    fn english(action: FooterAction) -> &'static str {
        match action {
            FooterAction::Navigate => "navigate",
            FooterAction::Open => "open",
            FooterAction::Help => "help",
            FooterAction::Refresh => "refresh",
            FooterAction::Quit => "quit",
            FooterAction::Select => "select",
            FooterAction::Back => "back",
            FooterAction::Page => "page",
            FooterAction::Search => "search",
            FooterAction::Filter => "filter",
            FooterAction::Sort => "sort",
            FooterAction::Edit => "edit",
            FooterAction::Change => "change",
            FooterAction::Apply => "apply",
            FooterAction::Revert => "revert",
            FooterAction::Cancel => "cancel",
            FooterAction::Discard => "discard",
            FooterAction::Dismiss => "dismiss",
        }
    }

    #[test]
    fn footer_is_contextual_sentence_grammar_not_a_dot_chain() {
        let footer = format_footer(FooterState::SettingsEdit, english);
        assert_eq!(
            footer,
            "↑↓ select  ←→ change  Enter edit  a apply  r revert"
        );
        assert!(!footer.contains('·'));
    }

    #[test]
    fn overlay_replaces_normal_footer() {
        assert_eq!(
            format_footer(FooterState::HelpOverlay, english),
            "Esc/?/q dismiss"
        );
    }
}
