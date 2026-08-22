//! Unicode display-cell-safe Human text helpers.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const ELLIPSIS: &str = "...";

/// Returns terminal display-cell width, not UTF-8 byte length.
#[must_use]
pub fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

/// Truncates without splitting a grapheme cluster.
#[must_use]
pub fn truncate_to_width(value: &str, maximum_width: usize) -> String {
    take_display_width(value, maximum_width)
}

/// Fits text into a display-cell width, adding an ASCII ellipsis when needed.
#[must_use]
pub fn fit_display_width(value: &str, width: usize) -> String {
    if display_width(value) <= width {
        value.to_owned()
    } else if width <= ELLIPSIS.len() {
        take_display_width(ELLIPSIS, width)
    } else {
        format!(
            "{}{}",
            take_display_width(value, width - ELLIPSIS.len()),
            ELLIPSIS
        )
    }
}

/// Right-pads text without exceeding its requested display-cell width.
#[must_use]
pub fn pad_display_width(value: &str, width: usize) -> String {
    let value = fit_display_width(value, width);
    format!(
        "{value}{:width$}",
        "",
        width = width.saturating_sub(display_width(&value))
    )
}

fn take_display_width(value: &str, width: usize) -> String {
    let mut result = String::new();
    let mut used = 0;
    for grapheme in value.graphemes(true) {
        let grapheme_width = display_width(grapheme);
        if used + grapheme_width > width {
            break;
        }
        result.push_str(grapheme);
        used += grapheme_width;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{display_width, fit_display_width, pad_display_width, truncate_to_width};

    #[test]
    fn cjk_and_combining_graphemes_are_cell_safe() {
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("诊"), 2);
        assert_eq!(truncate_to_width("e\u{301}诊断", 3), "e\u{301}诊");
        assert_eq!(fit_display_width("可靠性中心", 7), "可靠...");
        assert_eq!(display_width(&pad_display_width("诊", 4)), 4);
    }
}
