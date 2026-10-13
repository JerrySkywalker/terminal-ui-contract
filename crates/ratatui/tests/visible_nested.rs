#[path = "../examples/support/nested.rs"]
mod scene;
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn visible_nested_scene_shared_render_hit_focus_scroll_modal_and_resize() {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut scene = scene::NestedScene::default();
    terminal
        .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
        .unwrap();
    let rendered = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("Parent / "));
    assert!(rendered.contains("Nested / "));
    // Wide-character continuation cells have their own symbols in a raw buffer.
    assert!(
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() == "父")
    );
    assert!(
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|cell| cell.symbol() == "子")
    );
    for index in 0..7 {
        scene::scenario(&mut scene, index);
        terminal
            .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
            .unwrap();
    }
    terminal.backend_mut().resize(12, 6);
    scene.invalidate_geometry();
    terminal
        .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
        .unwrap();
    assert!(scene.child_rect.is_empty());
    assert!(
        scene
            .focus
            .current()
            .is_none_or(|path| path.parts().len() == 1)
    );
    terminal.backend_mut().resize(100, 30);
    scene.invalidate_geometry();
    terminal
        .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
        .unwrap();
    assert!(!scene.child_rect.is_empty());
}
