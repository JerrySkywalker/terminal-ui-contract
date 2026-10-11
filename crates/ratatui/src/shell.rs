//! Adaptive shell from actual layout rectangles; Compact renders one body pane.
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use terminal_ui_contract::{
    keymap::{EffectiveHint, InputMode},
    locale::{Locale, MessageKey, message},
    text::wrap_display_rows,
    viewport::{LayoutClass, ResponsivePolicy},
};

#[derive(Clone, Copy, Debug)]
pub struct MasterDetail {
    pub master: Option<Rect>,
    pub detail: Option<Rect>,
}
#[derive(Clone, Copy, Debug)]
pub struct ShellLayout {
    pub class: LayoutClass,
    pub header: Rect,
    pub footer: Rect,
    pub panes: MasterDetail,
}
impl ShellLayout {
    pub fn compute(area: Rect, policy: ResponsivePolicy, navigation_focused: bool) -> Self {
        let class = policy.classify(area.width, area.height);
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(area);
        let panes = match class {
            LayoutClass::Full => {
                let [master, detail] = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(policy.navigation_width),
                        Constraint::Min(0),
                    ])
                    .areas(body);
                MasterDetail {
                    master: Some(master),
                    detail: Some(detail),
                }
            }
            LayoutClass::Compact if navigation_focused => MasterDetail {
                master: Some(body),
                detail: None,
            },
            LayoutClass::Compact => MasterDetail {
                master: None,
                detail: Some(body),
            },
            LayoutClass::TooSmall => MasterDetail {
                master: None,
                detail: None,
            },
        };
        Self {
            class,
            header,
            footer,
            panes,
        }
    }
}

pub struct ShellChrome<'a> {
    pub identity: &'a str,
    pub focus: &'a str,
    pub mode: InputMode,
    pub locale: Locale,
    pub hints: &'a [EffectiveHint],
}
pub fn render_chrome(
    layout: ShellLayout,
    area: Rect,
    buffer: &mut Buffer,
    chrome: ShellChrome<'_>,
) {
    if layout.class == LayoutClass::TooSmall {
        let text = format!(
            "{}\n{}\nEsc / Ctrl+c",
            chrome.identity,
            message(chrome.locale, MessageKey::Resize)
        );
        let rows = wrap_display_rows(&text, area.width as usize);
        Paragraph::new(
            rows.into_iter()
                .map(ratatui::text::Line::raw)
                .collect::<Vec<_>>(),
        )
        .render(area, buffer);
        return;
    }
    Paragraph::new(format!(
        "{}\n{}: {} | {:?}",
        chrome.identity,
        message(chrome.locale, MessageKey::Focus),
        chrome.focus,
        chrome.mode
    ))
    .render(layout.header, buffer);
    let hints = chrome
        .hints
        .iter()
        .map(|hint| format!("{} {}", hint.keys, hint.label))
        .collect::<Vec<_>>()
        .join(" | ");
    let rows = wrap_display_rows(&hints, layout.footer.width as usize);
    Paragraph::new(
        rows.into_iter()
            .take(layout.footer.height as usize)
            .map(ratatui::text::Line::raw)
            .collect::<Vec<_>>(),
    )
    .render(layout.footer, buffer);
}
