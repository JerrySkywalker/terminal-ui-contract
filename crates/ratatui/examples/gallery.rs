//! TestBackend gallery export: synthetic source only, no actual terminal/provider/session.
use ratatui::{Terminal, backend::TestBackend};
use std::error::Error;
use terminal_ui_contract::{
    chrome::ColorPolicy,
    document::ViewDensity,
    keymap::{KeymapProfile, NavigationPolicy},
    locale::Locale,
    theme::{ColorLevel, ThemeResolver},
};
use terminal_ui_ratatui::synthetic::{Consumer, DemoApp};
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    let consumer = if value("--consumer") == Some("hookstat") {
        Consumer::HookStat
    } else {
        Consumer::TabBeacon
    };
    let locale = Locale::parse(value("--locale").unwrap_or("en-US")).ok_or("unsupported locale")?;
    let (width, height) = value("--size")
        .unwrap_or("100x32")
        .split_once('x')
        .ok_or("expected WIDTHxHEIGHT")?;
    let profile = if args.iter().any(|s| s == "--vim") {
        KeymapProfile::Vim
    } else {
        KeymapProfile::Standard
    };
    let theme = ThemeResolver::resolve(
        ColorPolicy::Auto,
        true,
        args.iter().any(|s| s == "--no-color") || std::env::var_os("NO_COLOR").is_some(),
        ColorLevel::Ansi16,
    );
    let mut app = DemoApp::new(consumer, profile, NavigationPolicy::Pane, locale, theme)?;
    app.select_screen(
        value("--screen")
            .unwrap_or(if consumer == Consumer::TabBeacon {
                "6"
            } else {
                "0"
            })
            .parse()?,
    );
    if args.iter().any(|s| s == "--raw") {
        app.view.density = ViewDensity::Raw;
    }
    // Export detail view in Compact as well as Full; keyboard nav entry is available in app.
    app.focus
        .reconcile(vec![terminal_ui_contract::focus::FocusPath::new(&[
            "details",
        ])]);
    let mut terminal = Terminal::new(TestBackend::new(width.parse()?, height.parse()?))?;
    terminal.draw(|frame| {
        if args.iter().any(|arg| arg == "--components") {
            terminal_ui_ratatui::synthetic::draw_component_gallery(
                consumer,
                locale,
                theme,
                frame.area(),
                frame.buffer_mut(),
            );
        } else {
            app.draw(frame.area(), frame.buffer_mut());
        }
    })?;
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area.height {
        let mut row = String::new();
        let mut skip = 0;
        for x in 0..buffer.area.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = buffer[(x, y)].symbol();
            row.push_str(symbol);
            skip = terminal_ui_contract::text::display_width(symbol).saturating_sub(1);
        }
        println!("{}", row.trim_end());
    }
    Ok(())
}
