//! Reflow once, render exactly those rows; no u16 offset truncation for long documents.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{
        Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    },
};
use terminal_ui_contract::{text::wrap_display_rows, viewport::BoundedViewport};

pub fn render_scroll_pane(
    text: &str,
    block: Block<'_>,
    area: Rect,
    buffer: &mut Buffer,
    viewport: &mut BoundedViewport,
) {
    let inner = block.inner(area);
    // Reserve one real display column for the scrollbar.
    let body = Rect {
        width: inner.width.saturating_sub(1),
        ..inner
    };
    let rows = wrap_display_rows(text, body.width as usize);
    viewport.reflow(rows.len(), body.height as usize);
    block
        .title_bottom(viewport.position_label())
        .render(area, buffer);
    let visible = rows
        .iter()
        .skip(viewport.offset())
        .take(viewport.visible_rows())
        .map(|s| ratatui::text::Line::raw(s.clone()))
        .collect::<Vec<_>>();
    Paragraph::new(Text::from(visible)).render(body, buffer);
    if inner.width > 0 && inner.height > 0 && viewport.max_offset() > 0 {
        let mut state = ScrollbarState::new(rows.len())
            .position(viewport.offset())
            .viewport_content_length(viewport.visible_rows());
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(inner, buffer, &mut state);
    }
}
