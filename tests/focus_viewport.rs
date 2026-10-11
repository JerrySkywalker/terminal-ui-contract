use terminal_ui_contract::{focus::*, viewport::*};

#[test]
fn focus_nested_cycle_resize_modal_and_stable_identity_are_view_only() {
    let nav = FocusPath::new(&["nav"]);
    let details = FocusPath::new(&["details"]);
    let editor = FocusPath::new(&["details", "editor"]);
    let mut focus = FocusState::default();
    focus.reconcile(vec![nav.clone(), details.clone(), editor.clone()]);
    assert_eq!(focus.current(), Some(&nav));
    focus.cycle(false);
    assert_eq!(focus.current(), Some(&details));
    focus.enter_child();
    assert_eq!(focus.current(), Some(&editor));
    focus.leave_child();
    assert_eq!(focus.current(), Some(&details));
    focus.focus(&editor);
    focus.reconcile(vec![nav.clone(), details.clone()]);
    assert_eq!(focus.current(), Some(&details));
    let modal = FocusPath::new(&["overlay"]);
    let confirm = FocusPath::new(&["overlay", "confirm"]);
    focus.push_modal(modal.clone());
    focus.reconcile(vec![nav.clone(), details.clone(), modal.clone()]);
    assert!(focus.focus(&nav).is_none());
    focus.cycle(false);
    assert_eq!(focus.current(), Some(&modal));
    focus.push_modal(confirm.clone());
    focus.reconcile(vec![nav.clone(), modal.clone(), confirm.clone()]);
    assert!(focus.focus(&modal).is_none());
    focus.pop_modal();
    assert_eq!(focus.current(), Some(&modal));
    focus.reconcile(vec![nav.clone(), details.clone(), modal]);
    focus.pop_modal();
    assert_eq!(focus.current(), Some(&details));
    const {
        assert!(!FOCUS_CHANGED_IMPLIES_PRODUCT_WRITE);
    }
    let mut selection = StableSelection::default();
    selection.select(&["a", "b", "c"], 1);
    selection.reconcile(&["c", "a", "b"]);
    assert_eq!(selection.selected(), Some(&"b"));
    assert_eq!(selection.index(), 2);
    selection.reconcile(&["a"]);
    assert_eq!(selection.selected(), Some(&"a"));
    selection.reconcile(&[]);
    assert_eq!(selection.selected(), None);
}

#[test]
fn viewport_exhaustive_clamp_page_home_end_and_large_values() {
    for content in 0..100 {
        for height in 0..40 {
            let mut viewport = BoundedViewport::default();
            viewport.reflow(content, height);
            for _ in 0..100 {
                viewport.page(1);
                assert!(viewport.offset() <= content.saturating_sub(height));
            }
            assert_eq!(viewport.offset(), content.saturating_sub(height));
            viewport.home();
            assert_eq!(viewport.offset(), 0);
            viewport.end();
            viewport.reflow(1, height);
            assert!(viewport.offset() <= 1usize.saturating_sub(height));
            viewport.lines(isize::MIN);
            assert_eq!(viewport.offset(), 0);
        }
    }
    let mut viewport = BoundedViewport::default();
    viewport.reflow(usize::MAX, 10);
    viewport.page(isize::MAX);
    assert!(viewport.offset() <= viewport.max_offset());
    viewport.page(isize::MIN);
    assert_eq!(viewport.offset(), 0);
    assert_eq!(
        ResponsivePolicy::default().classify(24, 10),
        LayoutClass::Compact
    );
    assert_eq!(
        ResponsivePolicy::default().classify(100, 32),
        LayoutClass::Full
    );
    assert_eq!(
        ResponsivePolicy::default().classify(23, 10),
        LayoutClass::TooSmall
    );
}
