//! First-party Ratatui thin components. Product facts and judgment remain supplied data.
use crate::{theme, viewport::render_scroll_pane};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Widget},
};
use terminal_ui_contract::{
    document::{FactFreshness, Severity, UiFact, Verdict},
    keymap::{KeymapRegistry, ResolveContext},
    locale::{Locale, MessageKey, message},
    theme::{ComponentToken, NeutralTheme, SemanticToken},
    viewport::BoundedViewport,
};

pub fn pane_block(title: &str, focused: bool, theme: NeutralTheme) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(theme::component(
            theme,
            if focused {
                ComponentToken::FocusedBorder
            } else {
                ComponentToken::Border
            },
        ))
        .title(if focused {
            format!("[FOCUS] {title}")
        } else {
            title.to_owned()
        })
}

pub struct StatusCard<'a> {
    pub fact: &'a UiFact,
    pub theme: NeutralTheme,
}
impl Widget for StatusCard<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let token = match self.fact.verdict {
            Verdict::Fail => SemanticToken::Failure,
            Verdict::Blocked | Verdict::Stale => SemanticToken::Attention,
            Verdict::Unknown | Verdict::Unproven => SemanticToken::Unknown,
            _ => SemanticToken::Information,
        };
        Paragraph::new(vec![
            Line::raw(format!(
                "[{}] {}",
                self.fact.verdict.marker(),
                self.fact.label
            )),
            Line::raw(self.fact.summary.as_str()),
            Line::raw(format!(
                "source={} | scope={}",
                self.fact.source, self.fact.scope
            )),
        ])
        .style(theme::semantic(self.theme, token))
        .block(pane_block("Status", false, self.theme))
        .render(area, buffer);
    }
}
pub struct MetricCard<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub unit: &'a str,
    pub theme: NeutralTheme,
}
impl Widget for MetricCard<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(format!("{} {}", self.value, self.unit))
            .block(pane_block(self.label, false, self.theme))
            .render(area, buffer);
    }
}

pub struct AttentionPanel<'a> {
    pub facts: &'a [UiFact],
    pub locale: Locale,
    pub theme: NeutralTheme,
}
impl Widget for AttentionPanel<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let lines = self
            .facts
            .iter()
            .filter(|fact| fact.severity != Severity::Information)
            .map(|fact| {
                Line::raw(format!(
                    "[{}] {}: {}",
                    fact.verdict.marker(),
                    fact.label,
                    fact.impact.as_str()
                ))
            })
            .collect::<Vec<_>>();
        Paragraph::new(lines)
            .block(pane_block(
                message(self.locale, MessageKey::Attention),
                false,
                self.theme,
            ))
            .render(area, buffer);
    }
}
pub struct FreshnessIndicator<'a> {
    pub fact: &'a UiFact,
    pub locale: Locale,
    pub theme: NeutralTheme,
}
impl Widget for FreshnessIndicator<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let freshness = match self.fact.freshness {
            FactFreshness::Current => MessageKey::Current,
            FactFreshness::Historical => MessageKey::Historical,
            FactFreshness::Unknown => MessageKey::Unknown,
        };
        Paragraph::new(format!(
            "{} | source={} | scope={}\nsampled_at={:?}",
            message(self.locale, freshness),
            self.fact.source,
            self.fact.scope,
            self.fact.sampled_at
        ))
        .block(pane_block(
            message(self.locale, MessageKey::Freshness),
            false,
            self.theme,
        ))
        .render(area, buffer);
    }
}
pub struct KeyValueList<'a> {
    pub title: &'a str,
    pub values: &'a [(&'a str, &'a str)],
    pub theme: NeutralTheme,
}
impl Widget for KeyValueList<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(
            self.values
                .iter()
                .map(|(key, value)| Line::raw(format!("{key}: {value}")))
                .collect::<Vec<_>>(),
        )
        .block(pane_block(self.title, false, self.theme))
        .render(area, buffer);
    }
}
pub struct EvidenceList<'a> {
    pub fact: &'a UiFact,
    pub theme: NeutralTheme,
}
impl Widget for EvidenceList<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(
            self.fact
                .evidence
                .iter()
                .map(|field| Line::raw(format!("{}: {}", field.label, field.value.as_str())))
                .collect::<Vec<_>>(),
        )
        .block(pane_block("Evidence (permitted)", false, self.theme))
        .render(area, buffer);
    }
}
pub struct DataTable<'a> {
    pub title: &'a str,
    pub headers: &'a [&'a str],
    pub rows: &'a [Vec<String>],
    pub theme: NeutralTheme,
}
impl Widget for DataTable<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let columns = self.headers.len().max(1);
        let widths = (0..columns)
            .map(|_| Constraint::Ratio(1, columns as u32))
            .collect::<Vec<_>>();
        Table::new(
            self.rows
                .iter()
                .map(|row| Row::new(row.iter().map(String::as_str))),
            widths,
        )
        .header(Row::new(self.headers.iter().copied()))
        .block(pane_block(self.title, false, self.theme))
        .render(area, buffer);
    }
}

pub struct ScrollPane<'a> {
    pub title: &'a str,
    pub text: &'a str,
    pub focused: bool,
    pub theme: NeutralTheme,
}

pub struct Sidebar<'a> {
    pub items: &'a [&'a str],
    pub selected: usize,
    pub focused: bool,
    pub theme: NeutralTheme,
}
impl Sidebar<'_> {
    pub fn render(
        self,
        area: Rect,
        buffer: &mut Buffer,
        viewport: &mut BoundedViewport,
        ensure_selection: bool,
    ) {
        let block = pane_block("Navigation", self.focused, self.theme);
        let inner = block.inner(area);
        viewport.reflow(self.items.len(), inner.height as usize);
        if ensure_selection {
            viewport.ensure_visible(self.selected);
        }
        let rows = self
            .items
            .iter()
            .enumerate()
            .skip(viewport.offset())
            .take(viewport.visible_rows())
            .map(|(i, item)| {
                Line::raw(format!(
                    "{} {item}",
                    if i == self.selected { ">" } else { " " }
                ))
            })
            .collect::<Vec<_>>();
        block
            .title_bottom(viewport.position_label())
            .render(area, buffer);
        Paragraph::new(rows).render(inner, buffer);
    }
}
impl ScrollPane<'_> {
    pub fn render(self, area: Rect, buffer: &mut Buffer, viewport: &mut BoundedViewport) {
        render_scroll_pane(
            self.text,
            pane_block(self.title, self.focused, self.theme),
            area,
            buffer,
            viewport,
        );
    }
}
pub struct Help<'a> {
    pub registry: &'a KeymapRegistry,
    pub context: &'a ResolveContext,
    pub locale: Locale,
    pub theme: NeutralTheme,
}
impl Help<'_> {
    pub fn render(self, area: Rect, buffer: &mut Buffer, viewport: &mut BoundedViewport) {
        let text = self
            .registry
            .effective_hints(self.context)
            .iter()
            .map(|hint| format!("{} — {} ({:?})", hint.keys, hint.label, hint.origin))
            .collect::<Vec<_>>()
            .join("\n");
        ScrollPane {
            title: message(self.locale, MessageKey::Help),
            text: &text,
            focused: true,
            theme: self.theme,
        }
        .render(area, buffer, viewport);
    }
}
pub struct Modal<'a> {
    pub title: &'a str,
    pub text: &'a str,
    pub theme: NeutralTheme,
}
impl Widget for Modal<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        ratatui::widgets::Clear.render(area, buffer);
        Paragraph::new(self.text)
            .block(pane_block(self.title, true, self.theme))
            .render(area, buffer);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MessagePanelKind {
    Loading,
    Empty,
    Error,
    Conflict,
}
pub struct MessagePanel<'a> {
    pub kind: MessagePanelKind,
    pub detail: &'a str,
    pub locale: Locale,
    pub theme: NeutralTheme,
}
impl Widget for MessagePanel<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let key = match self.kind {
            MessagePanelKind::Loading => MessageKey::Loading,
            MessagePanelKind::Empty => MessageKey::Empty,
            MessagePanelKind::Error => MessageKey::Error,
            MessagePanelKind::Conflict => MessageKey::Conflict,
        };
        Paragraph::new(self.detail)
            .block(pane_block(message(self.locale, key), false, self.theme))
            .render(area, buffer);
    }
}
