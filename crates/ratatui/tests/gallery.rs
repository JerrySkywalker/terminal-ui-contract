use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, widgets::Widget};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use terminal_ui_contract::{
    chrome::ColorPolicy, document::*, focus::FocusPath, input::*, keymap::*, locale::*, theme::*,
    viewport::BoundedViewport,
};
use terminal_ui_ratatui::{
    components::*,
    geometry::HitMap,
    synthetic::{Consumer, DemoApp, document},
    theme,
};
use terminal_ui_runtime::{refresh::*, snapshot::*};
fn app(consumer: Consumer, profile: KeymapProfile) -> DemoApp {
    DemoApp::new(
        consumer,
        profile,
        NavigationPolicy::Pane,
        Locale::EnUs,
        ThemeResolver::resolve(ColorPolicy::Always, true, true, ColorLevel::TrueColor),
    )
    .unwrap()
}
fn key(text: &str) -> InputEvent {
    InputEvent::Key(KeyEvent {
        chord: parse_chord(text).unwrap(),
        kind: InputKind::Press,
    })
}
fn mouse(x: u16, y: u16, kind: MouseKind) -> InputEvent {
    InputEvent::Mouse(MouseEvent {
        column: x,
        row: y,
        kind,
        modifiers: Modifiers::NONE,
    })
}
fn draw(app: &mut DemoApp, width: u16, height: u16) -> (HitMap, Buffer) {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    let map = app.draw(buffer.area, &mut buffer);
    (map, buffer)
}
fn text(buffer: &Buffer) -> String {
    let mut result = String::new();
    for y in buffer.area.y..buffer.area.bottom() {
        let mut skip = 0;
        for x in buffer.area.x..buffer.area.right() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = buffer[(x, y)].symbol();
            result.push_str(symbol);
            skip = terminal_ui_contract::text::display_width(symbol).saturating_sub(1);
        }
        result.push('\n');
    }
    result
}

#[test]
fn two_consumer_pointer_focus_never_dispatches_writer_and_custom_map_cannot_grant_permission() {
    for consumer in [Consumer::TabBeacon, Consumer::HookStat] {
        let mut app = app(consumer, KeymapProfile::Standard);
        if consumer == Consumer::TabBeacon {
            app.select_screen(1);
        }
        app.dirty = true;
        app.conflict = true;
        let (map, _) = draw(&mut app, 100, 32);
        let screen = app.navigation.current();
        app.handle(
            mouse(30, 8, MouseKind::Press(MouseButton::Primary)),
            Some(&map),
        );
        assert_eq!(app.focus.current(), Some(&FocusPath::new(&["details"])));
        assert_eq!(
            (
                app.navigation.current(),
                app.dirty,
                app.conflict,
                app.writer_calls,
                app.write_requests
            ),
            (screen, true, true, 0, 0)
        );
        let (map, _) = draw(&mut app, 100, 32);
        app.handle(
            mouse(2, 8, MouseKind::Press(MouseButton::Primary)),
            Some(&map),
        );
        let before = app.focus.current().cloned();
        app.handle(mouse(30, 8, MouseKind::Move), Some(&map));
        app.handle(mouse(30, 8, MouseKind::ScrollDown), Some(&map));
        assert_eq!(app.focus.current(), before.as_ref());
        assert_eq!((app.writer_calls, app.write_requests), (0, 0));
        if consumer == Consumer::TabBeacon {
            let map=app.keymap.overlay_toml("schema_version=1\n[[bindings]]\ncontext='settings'\nkeys=['x']\naction='synthetic.apply'").unwrap();
            app.set_keymap(map);
            app.handle(key("x"), None);
            assert_eq!((app.write_requests, app.writer_calls), (1, 0));
            app.write_authorized = true;
            app.write_confirmed = true;
            app.handle(key("x"), None);
            assert_eq!(app.writer_calls, 1); // positive fake-writer control
            let (map, _) = draw(&mut app, 100, 32);
            app.handle(
                mouse(30, 8, MouseKind::Press(MouseButton::Primary)),
                Some(&map),
            );
            app.handle(key("Tab"), None);
            app.handle(key("F2"), None);
            app.handle(key("?"), None);
            let (map, _) = draw(&mut app, 100, 32);
            app.handle(
                mouse(2, 8, MouseKind::Press(MouseButton::Primary)),
                Some(&map),
            );
            app.handle(key("x"), None);
            assert_eq!(app.writer_calls, 1);
            assert_eq!(app.write_requests, 2);
            assert!(app.dirty && app.conflict);
        }
    }
}
#[test]
fn nine_screens_two_densities_locale_no_color_narrow_and_footer_dispatch_parity() {
    for consumer in [Consumer::TabBeacon, Consumer::HookStat] {
        for locale in [Locale::EnUs, Locale::ZhCn] {
            let mut app = app(consumer, KeymapProfile::Standard);
            app.locale = locale;
            for screen in 0..consumer.screens().len() {
                app.select_screen(screen);
                for (width, height) in [(100, 32), (40, 16), (24, 10), (18, 6)] {
                    let (_, buffer) = draw(&mut app, width, height);
                    assert!(!text(&buffer).trim().is_empty());
                    assert!(
                        buffer
                            .content
                            .iter()
                            .all(|cell| cell.fg == ratatui::style::Color::Reset
                                && cell.bg == ratatui::style::Color::Reset)
                    );
                }
            }
            let (_, buffer) = draw(&mut app, 100, 32);
            assert!(text(&buffer).contains(message(locale, MessageKey::Focus)));
            assert!(text(&buffer).contains("Normal"));
            let current = app.document.clone();
            let queries = app.query_requests;
            let map=app.keymap.overlay_toml("schema_version=1\n[[unbind]]\ncontext='global'\nkeys=['F2']\n[[bindings]]\ncontext='global'\nkeys=['F3']\naction='ui.view.toggle_density'").unwrap();
            app.set_keymap(map);
            assert_eq!(app.handle(key("F2"), None), Resolution::NoBinding);
            app.handle(key("F3"), None);
            assert_eq!(app.view.density, ViewDensity::Raw);
            assert!(Arc::ptr_eq(&app.document, &current));
            assert_eq!(app.query_requests, queries);
            let hints = app.keymap.effective_hints(&app.context());
            assert!(
                hints
                    .iter()
                    .any(|h| h.keys == "F3" && h.action_id == "ui.view.toggle_density")
            );
            assert!(!hints.iter().any(|h| h.keys == "F2"));
            let (_, rendered) = draw(&mut app, 100, 32);
            let footer = text(&rendered)
                .lines()
                .skip(30)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(footer.contains("F3"));
            assert!(!footer.contains("F2"));
        }
    }
}
#[test]
fn compact_keyboard_entry_editor_text_and_overlay_scroll_trap() {
    let mut app = app(Consumer::TabBeacon, KeymapProfile::Vim);
    draw(&mut app, 24, 10);
    app.handle(key("Enter"), None);
    let (_, buffer) = draw(&mut app, 24, 10);
    assert_eq!(app.focus.current(), Some(&FocusPath::new(&["details"])));
    assert!(text(&buffer).contains("FOCUS"));
    app.handle(key("Tab"), None);
    draw(&mut app, 24, 10);
    assert_eq!(app.focus.current(), Some(&FocusPath::new(&["navigation"])));
    app.handle(key("/"), None);
    draw(&mut app, 100, 32);
    assert_eq!(app.mode, InputMode::Insert);
    for value in ["q", "r", "m", "1", "2", "3"] {
        app.handle(key(value), None);
    }
    app.handle(InputEvent::Paste("中文".into()), None);
    assert_eq!(app.editor_text, "qrm123中文");
    assert_eq!(app.query_requests, 0);
    assert_eq!(app.writer_calls, 0);
    app.handle(key("Esc"), None);
    assert_eq!(app.mode, InputMode::Normal);
    app.handle(key("i"), None);
    assert_eq!(app.mode, InputMode::Insert);
    app.handle(key("Esc"), None);
    app.handle(key("Esc"), None);
    draw(&mut app, 100, 32);
    app.handle(key("?"), None);
    let (map, _) = draw(&mut app, 100, 32);
    let focus = app.focus.current().cloned();
    let queries = app.query_requests;
    for value in ["q", "m", "3", "F5"] {
        assert_eq!(app.handle(key(value), None), Resolution::Consumed);
    }
    app.handle(mouse(1, 5, MouseKind::ScrollDown), Some(&map));
    assert_eq!(app.focus.current(), focus.as_ref());
    assert_eq!(app.query_requests, queries);
    for _ in 0..100 {
        app.handle(key("PageDown"), None);
    }
    let viewport = app.viewports.get(&FocusPath::new(&["help"])).unwrap();
    assert!(viewport.offset() <= viewport.max_offset());
}
#[test]
fn component_gallery_covers_cards_tables_evidence_modal_messages_and_color_levels() {
    let document = document(Consumer::TabBeacon, 6, 1);
    let fact = &document.facts()[0];
    let area = Rect::new(0, 0, 100, 8);
    for level in [
        ColorLevel::None,
        ColorLevel::Ansi16,
        ColorLevel::Ansi256,
        ColorLevel::TrueColor,
    ] {
        let theme = NeutralTheme { color_level: level };
        let mut buffer = Buffer::empty(area);
        StatusCard { fact, theme }.render(area, &mut buffer);
        assert!(text(&buffer).contains("BLOCKED"));
        MetricCard {
            label: "Synthetic events",
            value: "42",
            unit: "events",
            theme,
        }
        .render(area, &mut buffer);
        AttentionPanel {
            facts: document.facts(),
            locale: Locale::ZhCn,
            theme,
        }
        .render(area, &mut buffer);
        FreshnessIndicator {
            fact,
            locale: Locale::ZhCn,
            theme,
        }
        .render(area, &mut buffer);
        KeyValueList {
            title: "Values",
            values: &[("source", "synthetic"), ("scope", "fixture")],
            theme,
        }
        .render(area, &mut buffer);
        EvidenceList { fact, theme }.render(area, &mut buffer);
        assert!(text(&buffer).contains("Synthetic count"));
        DataTable {
            title: "Data",
            headers: &["ID", "Value"],
            rows: &[vec!["one".into(), "1".into()]],
            theme,
        }
        .render(area, &mut buffer);
        assert!(text(&buffer).contains("one"));
        Modal {
            title: "Confirm (no writer)",
            text: "explicit decision required",
            theme,
        }
        .render(area, &mut buffer);
        for kind in [
            MessagePanelKind::Loading,
            MessagePanelKind::Empty,
            MessagePanelKind::Error,
            MessagePanelKind::Conflict,
        ] {
            MessagePanel {
                kind,
                detail: "synthetic state",
                locale: Locale::ZhCn,
                theme,
            }
            .render(area, &mut buffer);
        }
        assert_eq!(
            theme::semantic(theme, SemanticToken::Failure).fg,
            match level {
                ColorLevel::None => None,
                ColorLevel::Ansi16 => Some(ratatui::style::Color::Red),
                ColorLevel::Ansi256 => Some(ratatui::style::Color::Indexed(160)),
                ColorLevel::TrueColor => Some(ratatui::style::Color::Rgb(215, 70, 70)),
            }
        );
    }
}
struct SlowDocument {
    count: Arc<AtomicUsize>,
    started: SyncSender<()>,
}

struct ImmediateDocument;
impl ReadOnlyCollector<(Consumer, usize), UiDocument> for ImmediateDocument {
    fn collect(
        &mut self,
        request: &RefreshRequest<(Consumer, usize)>,
        control: &CollectionControl,
    ) -> Result<Collected<UiDocument>, CollectError> {
        control.check()?;
        Ok(Collected {
            value: document(request.reason.0, request.reason.1, 0),
            empty: false,
            metadata: SnapshotMetadata {
                source: request.reason.0.name().into(),
                scope: "synthetic immutable facts".into(),
                sampled_at: SystemTime::UNIX_EPOCH,
                freshness: Freshness::Unknown,
            },
        })
    }
}
#[test]
fn both_consumers_accept_same_runtime_api_preserving_identity_and_draft() {
    for consumer in [Consumer::TabBeacon, Consumer::HookStat] {
        let mut app = app(consumer, KeymapProfile::Standard);
        app.dirty = true;
        app.conflict = true;
        let (_, _) = draw(&mut app, 100, 32);
        let focus = app.focus.current().cloned();
        app.selection.select(
            &app.document
                .facts()
                .iter()
                .map(|fact| fact.id.clone())
                .collect::<Vec<_>>(),
            2,
        );
        let selected = app.selection.selected().cloned();
        let mut runtime = RefreshController::spawn(
            ImmediateDocument,
            RefreshPolicy {
                task_timeout: Duration::from_secs(2),
                poll_interval: Duration::from_secs(1),
                stale_after: Duration::from_secs(5),
            },
        )
        .unwrap();
        for iteration in 0..2 {
            runtime.request((consumer, 0), RequestKind::Manual).unwrap();
            let started = Instant::now();
            let outcome = loop {
                let outcome = runtime.poll();
                if outcome != RefreshPoll::Pending {
                    break outcome;
                }
                assert!(started.elapsed() < Duration::from_secs(2));
                thread::yield_now();
            };
            assert!(
                matches!(outcome,RefreshPoll::Accepted { changed,.. } if changed==(iteration==0))
            );
            let snapshot = runtime.snapshot().unwrap();
            let mut facts = snapshot.value().facts().to_vec();
            facts.reverse();
            app.accept_document(Arc::new(
                UiDocument::new(snapshot.revision(), snapshot.value().sampled_at(), facts).unwrap(),
            ));
            assert_eq!(app.selection.selected(), selected.as_ref());
            assert_eq!(app.focus.current(), focus.as_ref());
            assert!(app.dirty && app.conflict);
            assert_eq!(app.writer_calls, 0);
        }
        runtime.shutdown(Duration::from_secs(1)).unwrap();
    }
}
impl ReadOnlyCollector<usize, UiDocument> for SlowDocument {
    fn collect(
        &mut self,
        request: &RefreshRequest<usize>,
        control: &CollectionControl,
    ) -> Result<Collected<UiDocument>, CollectError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.started.send(()).unwrap();
        control.wait_for(Duration::from_secs(2))?;
        Ok(Collected {
            value: document(Consumer::TabBeacon, request.reason, 7),
            empty: false,
            metadata: SnapshotMetadata {
                source: "synthetic document".into(),
                scope: "fixture only".into(),
                sampled_at: SystemTime::UNIX_EPOCH,
                freshness: Freshness::Unknown,
            },
        })
    }
}
#[test]
fn foreground_keys_and_frames_continue_during_slow_read_only_snapshot() {
    let count = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = sync_channel(1);
    let mut runtime = RefreshController::spawn(
        SlowDocument {
            count: count.clone(),
            started: tx,
        },
        RefreshPolicy {
            task_timeout: Duration::from_secs(5),
            poll_interval: Duration::from_secs(3),
            stale_after: Duration::from_secs(10),
        },
    )
    .unwrap();
    let mut app = app(Consumer::TabBeacon, KeymapProfile::Standard);
    app.dirty = true;
    app.conflict = true;
    runtime.request(6, RequestKind::Manual).unwrap();
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    let start = Instant::now();
    for _ in 0..100 {
        app.handle(key("F2"), None);
        terminal
            .draw(|f| {
                app.draw(f.area(), f.buffer_mut());
            })
            .unwrap();
        assert_eq!(runtime.poll(), RefreshPoll::Pending);
    }
    let elapsed = start.elapsed();
    println!("synthetic 100 input+TestBackend frames while 2s collector: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(2));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(app.query_requests, 0);
    assert!(app.dirty && app.conflict);
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if matches!(runtime.poll(), RefreshPoll::Accepted { .. }) {
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    let snapshot = runtime.snapshot().unwrap();
    app.accept_document(Arc::new(
        UiDocument::new(
            snapshot.revision(),
            snapshot.value().sampled_at(),
            snapshot.value().facts().to_vec(),
        )
        .unwrap(),
    ));
    assert!(app.dirty && app.conflict);
    assert_eq!(app.writer_calls, 0);
    assert_eq!(app.document.revision(), snapshot.revision());
    runtime.shutdown(Duration::from_secs(1)).unwrap();
}
#[test]
fn product_allowlist_keeps_synthetic_private_fields_out_of_both_projections() {
    struct Observation {
        allowed_count: usize,
        private_payload: String,
        command: String,
    }
    let observation = Observation {
        allowed_count: 4,
        private_payload: "SYNTHETIC_PRIVATE_PAYLOAD".into(),
        command: "SYNTHETIC_COMMAND_NEVER_FORWARDED".into(),
    };
    let mut facts = document(Consumer::HookStat, 0, 1).facts().to_vec();
    facts[0].evidence = vec![EvidenceField {
        label: "Allowed count".into(),
        value: PermittedText::from_product_redaction(observation.allowed_count.to_string()),
    }];
    let document = UiDocument::new(1, SystemTime::UNIX_EPOCH, facts).unwrap();
    for density in [ViewDensity::Smart, ViewDensity::Raw] {
        let text = document.project(density).lines().join("\n");
        assert!(!text.contains(&observation.private_payload));
        assert!(!text.contains(&observation.command));
    }
    let mut viewport = BoundedViewport::default();
    viewport.reflow(10, 4);
    viewport.end();
    assert_eq!(viewport.offset(), 6);
}
