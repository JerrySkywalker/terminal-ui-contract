use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders},
};
use std::{cell::RefCell, io, rc::Rc};
use terminal_ui_contract::{focus::*, input::*, text::*, viewport::*};
use terminal_ui_ratatui::{geometry::*, terminal::*, viewport::*};

fn region(path: &[&str], rect: Rect, z: u16, focusable: bool, scrollable: bool) -> HitRegion {
    HitRegion {
        path: FocusPath::new(path),
        rect,
        clip: Rect::new(0, 0, 100, 32),
        z,
        enabled: true,
        focusable,
        scrollable,
    }
}
fn pointer(column: u16, row: u16, kind: MouseKind) -> MouseEvent {
    MouseEvent {
        column,
        row,
        kind,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn two_consumers_click_hover_wheel_nested_clip_and_modal_are_view_only() {
    for (nav, details, nested) in [
        ("navigation", "doctor", "evidence"),
        ("events", "handlers", "details"),
    ] {
        let map = HitMap::new(
            4,
            vec![
                region(&[nav], Rect::new(0, 0, 21, 30), 0, true, true),
                region(&[details], Rect::new(21, 0, 79, 30), 0, true, true),
                region(&[details, nested], Rect::new(25, 5, 65, 20), 0, true, true),
                region(
                    &[details, nested, "label"],
                    Rect::new(25, 5, 65, 1),
                    0,
                    false,
                    false,
                ),
                region(&["status"], Rect::new(0, 30, 100, 2), 0, true, false),
            ],
        );
        let mut focus = FocusState::default();
        map.reconcile_focus(&mut focus);
        let selected_page = "synthetic";
        let dirty = true;
        let conflict = true;
        let write_calls = 0;
        assert!(matches!(
            map.pointer(
                pointer(30, 8, MouseKind::Press(MouseButton::Primary)),
                4,
                &mut focus
            ),
            PointerEffect::Focus(_)
        ));
        assert_eq!(focus.current(), Some(&FocusPath::new(&[details, nested])));
        map.pointer(
            pointer(2, 2, MouseKind::Press(MouseButton::Primary)),
            4,
            &mut focus,
        );
        assert_eq!(focus.current(), Some(&FocusPath::new(&[nav])));
        assert_eq!(
            map.pointer(pointer(30, 5, MouseKind::ScrollDown), 4, &mut focus),
            PointerEffect::Scroll {
                path: FocusPath::new(&[details, nested]),
                lines: 3
            }
        );
        assert_eq!(focus.current(), Some(&FocusPath::new(&[nav])));
        map.pointer(pointer(30, 8, MouseKind::Move), 4, &mut focus);
        assert_eq!(focus.current(), Some(&FocusPath::new(&[nav])));
        assert_eq!(
            map.pointer(pointer(30, 31, MouseKind::ScrollDown), 4, &mut focus),
            PointerEffect::Consumed
        );
        assert_eq!(
            map.pointer(
                pointer(30, 8, MouseKind::Press(MouseButton::Primary)),
                5,
                &mut focus
            ),
            PointerEffect::StaleGeometry
        );
        let overlay = HitMap::new(
            5,
            vec![
                region(&[nav], Rect::new(0, 0, 21, 30), 0, true, true),
                region(&["overlay"], Rect::new(30, 10, 40, 10), 10, true, true),
            ],
        );
        focus.push_modal(FocusPath::new(&["overlay"]));
        overlay.reconcile_focus(&mut focus);
        for kind in [
            MouseKind::ScrollDown,
            MouseKind::Press(MouseButton::Primary),
        ] {
            assert_eq!(
                overlay.pointer(pointer(2, 2, kind), 5, &mut focus),
                PointerEffect::Consumed
            );
        }
        assert_eq!(focus.current(), Some(&FocusPath::new(&["overlay"])));
        assert_eq!(
            (selected_page, dirty, conflict, write_calls),
            ("synthetic", true, true, 0)
        );
    }
}

#[test]
fn topmost_deepest_and_visible_ancestor_rules() {
    let parent = region(&["details"], Rect::new(0, 0, 10, 10), 0, true, true);
    let child = region(
        &["details", "label"],
        Rect::new(5, 5, 20, 20),
        0,
        false,
        false,
    );
    let top = region(&["overlay"], Rect::new(8, 8, 2, 2), 10, true, false);
    let map = HitMap::new(1, vec![parent, child, top]);
    let mut focus = FocusState::default();
    map.reconcile_focus(&mut focus);
    assert_eq!(
        map.pointer(pointer(6, 6, MouseKind::ScrollDown), 1, &mut focus),
        PointerEffect::Scroll {
            path: FocusPath::new(&["details"]),
            lines: 3
        }
    );
    assert_eq!(
        map.pointer(pointer(15, 15, MouseKind::ScrollDown), 1, &mut focus),
        PointerEffect::Consumed
    );
    assert_eq!(
        map.pointer(pointer(8, 8, MouseKind::ScrollDown), 1, &mut focus),
        PointerEffect::Consumed
    );
    map.pointer(
        pointer(8, 8, MouseKind::Press(MouseButton::Primary)),
        1,
        &mut focus,
    );
    assert_eq!(focus.current(), Some(&FocusPath::new(&["overlay"])));
}

#[test]
fn unicode_rows_match_rendered_viewport_last_row_and_reflow() {
    for (width, height) in [(24, 10), (40, 16), (100, 32), (5, 4), (0, 0)] {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        let mut viewport = BoundedViewport::default();
        let text = format!("{}\nLAST", "诊断 e\u{301} 👩‍💻 long\n".repeat(140));
        render_scroll_pane(
            &text,
            Block::default().borders(Borders::ALL),
            area,
            &mut buffer,
            &mut viewport,
        );
        for _ in 0..100 {
            viewport.page(1);
        }
        render_scroll_pane(
            &text,
            Block::default().borders(Borders::ALL),
            area,
            &mut buffer,
            &mut viewport,
        );
        assert!(viewport.offset() <= viewport.max_offset());
        viewport.end();
        render_scroll_pane(
            &text,
            Block::default().borders(Borders::ALL),
            area,
            &mut buffer,
            &mut viewport,
        );
        if width >= 24 {
            let rendered = buffer
                .content
                .iter()
                .map(|c| c.symbol())
                .collect::<String>();
            assert!(rendered.contains("LAST"));
        }
        render_scroll_pane(
            "短",
            Block::default().borders(Borders::ALL),
            area,
            &mut buffer,
            &mut viewport,
        );
        assert_eq!(viewport.offset(), 0);
    }
    for width in 1..20 {
        for row in wrap_display_rows("诊断e\u{301}👩‍💻\t文", width) {
            assert!(display_width(&row) <= width);
            assert!(!row.contains('\t'));
        }
    }
    assert_eq!(wrap_display_rows("e\u{301}诊", 1), vec!["e\u{301}", "?"]);
    // Offset exceeds Paragraph's u16 scroll range; our sliced rows remain reachable.
    let text = format!("{}END", "x\n".repeat(70_000));
    let mut viewport = BoundedViewport::default();
    viewport.reflow(70_001, 4);
    viewport.end();
    let area = Rect::new(0, 0, 20, 6);
    let mut buffer = Buffer::empty(area);
    render_scroll_pane(
        &text,
        Block::default().borders(Borders::ALL),
        area,
        &mut buffer,
        &mut viewport,
    );
    assert!(
        buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
            .contains("END")
    );
}

#[derive(Clone)]
struct FakeOps {
    log: Rc<RefCell<Vec<(TerminalStep, bool)>>>,
    fail_enter: Option<TerminalStep>,
    fail_exit_once: Option<TerminalStep>,
}
impl TerminalOps for FakeOps {
    fn apply(&mut self, step: TerminalStep, enter: bool) -> io::Result<()> {
        self.log.borrow_mut().push((step, enter));
        if enter && self.fail_enter == Some(step) {
            self.fail_enter = None;
            return Err(io::Error::other("injected entry"));
        }
        if !enter && self.fail_exit_once == Some(step) {
            self.fail_exit_once = None;
            return Err(io::Error::other("injected cleanup"));
        }
        Ok(())
    }
}
#[test]
fn terminal_partial_entry_cleanup_retry_and_unwind() {
    for step in [
        TerminalStep::Raw,
        TerminalStep::Alternate,
        TerminalStep::Cursor,
        TerminalStep::Mouse,
        TerminalStep::Paste,
    ] {
        let log = Rc::new(RefCell::new(vec![]));
        assert!(
            TerminalGuard::enter(
                FakeOps {
                    log: log.clone(),
                    fail_enter: Some(step),
                    fail_exit_once: None
                },
                true
            )
            .is_err()
        );
        let enter = log
            .borrow()
            .iter()
            .filter(|(_, e)| *e)
            .map(|(s, _)| *s)
            .collect::<Vec<_>>();
        let exits = log
            .borrow()
            .iter()
            .filter(|(_, e)| !*e)
            .map(|(s, _)| *s)
            .collect::<Vec<_>>();
        assert_eq!(enter.into_iter().rev().collect::<Vec<_>>(), exits);
    }
    let log = Rc::new(RefCell::new(vec![]));
    let mut guard = TerminalGuard::enter(
        FakeOps {
            log: log.clone(),
            fail_enter: None,
            fail_exit_once: Some(TerminalStep::Mouse),
        },
        true,
    )
    .unwrap();
    assert!(guard.finish().is_err());
    assert!(guard.finish().is_ok());
    drop(guard);
    assert_eq!(
        log.borrow()
            .iter()
            .filter(|event| **event == (TerminalStep::Mouse, false))
            .count(),
        2
    );
    let log = Rc::new(RefCell::new(vec![]));
    let unwind_log = log.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _guard = TerminalGuard::enter(
            FakeOps {
                log: unwind_log,
                fail_enter: None,
                fail_exit_once: None,
            },
            true,
        )
        .unwrap();
        panic!("synthetic unwind");
    }));
    assert!(result.is_err());
    assert!(log.borrow().contains(&(TerminalStep::Raw, false)));
    assert!(log.borrow().contains(&(TerminalStep::Mouse, false)));
}
