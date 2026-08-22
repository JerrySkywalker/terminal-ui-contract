//! Semantic Human chrome tokens independent of a concrete terminal renderer.

/// A user-facing color policy shared by terminal consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorPolicy {
    /// Enable color only for an interactive terminal that did not opt out.
    Auto,
    /// Enable color even for redirected Human output unless `NO_COLOR` is set.
    Always,
    /// Use terminal defaults only.
    Never,
}

/// A chrome role which a consumer maps to its terminal library's style type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChromeToken {
    Base,
    HeaderTitle,
    Border,
    Footer,
    CurrentScreen,
    Overlay,
}

/// Returns whether the shared Human chrome may use color.
#[must_use]
pub const fn color_enabled(policy: ColorPolicy, is_terminal: bool, no_color_is_set: bool) -> bool {
    match policy {
        ColorPolicy::Never => false,
        ColorPolicy::Auto => is_terminal && !no_color_is_set,
        ColorPolicy::Always => !no_color_is_set,
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorPolicy, color_enabled};

    #[test]
    fn no_color_is_a_global_human_chrome_opt_out() {
        assert!(!color_enabled(ColorPolicy::Always, true, true));
        assert!(!color_enabled(ColorPolicy::Auto, true, true));
        assert!(!color_enabled(ColorPolicy::Never, true, false));
        assert!(color_enabled(ColorPolicy::Auto, true, false));
        assert!(color_enabled(ColorPolicy::Always, false, false));
    }
}
