//! Neutral semantic/component/surface tokens; no future CSIS theme dependency.
use crate::chrome::{ColorPolicy, color_enabled};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorLevel {
    None,
    Ansi16,
    Ansi256,
    TrueColor,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticToken {
    Information,
    Attention,
    Failure,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentToken {
    Border,
    FocusedBorder,
    Title,
    Selection,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceToken {
    Base,
    Overlay,
    Footer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeutralTheme {
    pub color_level: ColorLevel,
}
pub struct ThemeResolver;
impl ThemeResolver {
    pub fn resolve(
        policy: ColorPolicy,
        is_terminal: bool,
        no_color: bool,
        capability: ColorLevel,
    ) -> NeutralTheme {
        NeutralTheme {
            color_level: if color_enabled(policy, is_terminal, no_color) {
                capability
            } else {
                ColorLevel::None
            },
        }
    }
}
