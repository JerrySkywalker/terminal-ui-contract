//! Neutral backend color fallback; all states also have text markers.
use ratatui::style::{Color, Style};
use terminal_ui_contract::theme::{
    ColorLevel, ComponentToken, NeutralTheme, SemanticToken, SurfaceToken,
};

pub fn semantic(theme: NeutralTheme, token: SemanticToken) -> Style {
    let (ansi, index, rgb) = match token {
        SemanticToken::Information => (Color::Blue, 33, (80, 150, 220)),
        SemanticToken::Attention => (Color::Yellow, 178, (210, 170, 60)),
        SemanticToken::Failure => (Color::Red, 160, (215, 70, 70)),
        SemanticToken::Unknown => (Color::Gray, 245, (140, 140, 140)),
    };
    color_style(theme, ansi, index, rgb)
}
pub fn component(theme: NeutralTheme, token: ComponentToken) -> Style {
    let color = match token {
        ComponentToken::FocusedBorder | ComponentToken::Selection => Color::Cyan,
        ComponentToken::Title => Color::White,
        ComponentToken::Border => Color::Gray,
    };
    color_style(
        theme,
        color,
        if color == Color::Cyan { 44 } else { 245 },
        if color == Color::Cyan {
            (80, 185, 185)
        } else {
            (150, 150, 150)
        },
    )
}
pub fn surface(theme: NeutralTheme, _token: SurfaceToken) -> Style {
    if theme.color_level == ColorLevel::None {
        Style::default()
    } else {
        Style::default().bg(Color::Reset)
    }
}
fn color_style(theme: NeutralTheme, ansi: Color, index: u8, rgb: (u8, u8, u8)) -> Style {
    match theme.color_level {
        ColorLevel::None => Style::default(),
        ColorLevel::Ansi16 => Style::default().fg(ansi),
        ColorLevel::Ansi256 => Style::default().fg(Color::Indexed(index)),
        ColorLevel::TrueColor => Style::default().fg(Color::Rgb(rgb.0, rgb.1, rgb.2)),
    }
}
