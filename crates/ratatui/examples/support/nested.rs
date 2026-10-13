//! Example/test-only visible scene; uses the same rendered geometry for HitMap.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Clear, Paragraph, Widget},
};
use terminal_ui_contract::{
    focus::{FocusPath, FocusState},
    input::{Modifiers, MouseButton, MouseEvent, MouseKind},
    theme::{ColorLevel, NeutralTheme},
    viewport::BoundedViewport,
};
use terminal_ui_ratatui::{
    components::pane_block,
    geometry::{HitMap, HitRegion, PointerEffect},
    viewport::render_scroll_pane,
};

pub struct NestedScene {
    pub focus: FocusState,
    pub parent: BoundedViewport,
    pub child: BoundedViewport,
    pub hits: HitMap,
    pub generation: u64,
    pub modal: bool,
    pub parent_rect: Rect,
    pub child_rect: Rect,
}
impl Default for NestedScene {
    fn default() -> Self {
        Self {
            focus: FocusState::default(),
            parent: BoundedViewport::default(),
            child: BoundedViewport::default(),
            hits: HitMap::new(0, vec![]),
            generation: 0,
            modal: false,
            parent_rect: Rect::default(),
            child_rect: Rect::default(),
        }
    }
}
fn region(
    parts: &[&str],
    rect: Rect,
    clip: Rect,
    z: u16,
    focusable: bool,
    scrollable: bool,
) -> HitRegion {
    HitRegion {
        path: FocusPath::new(parts),
        rect,
        clip,
        z,
        enabled: true,
        focusable,
        scrollable,
    }
}
impl NestedScene {
    pub fn invalidate_geometry(&mut self) {
        self.generation += 1;
        self.hits = HitMap::new(self.generation, vec![]);
    }
    pub fn set_modal(&mut self, open: bool) {
        if self.modal == open {
            return;
        }
        self.modal = open;
        if open {
            self.focus.push_modal(FocusPath::new(&["modal"]));
        } else {
            self.focus.pop_modal();
        }
        self.invalidate_geometry();
    }
    pub fn pointer(&mut self, event: MouseEvent) -> PointerEffect {
        let effect = self.hits.pointer(event, self.generation, &mut self.focus);
        if let PointerEffect::Scroll { path, lines } = &effect {
            if path == &FocusPath::new(&["parent", "child"]) {
                self.child.lines(*lines);
            } else if path == &FocusPath::new(&["parent"]) {
                self.parent.lines(*lines);
            }
        }
        effect
    }
    pub fn draw(&mut self, area: Rect, buffer: &mut Buffer) {
        let theme = NeutralTheme {
            color_level: ColorLevel::None,
        };
        self.parent_rect = area;
        let inner = pane_block("", false, theme).inner(area);
        // Actual child occupies a subset of the parent body. Rendering is clipped too.
        self.child_rect = if inner.width >= 16 && inner.height >= 8 {
            Rect::new(
                inner.x + inner.width / 3,
                inner.y + 2,
                inner.width * 2 / 3,
                inner.height.saturating_sub(3),
            )
            .intersection(inner)
        } else {
            Rect::default()
        };
        let parent_path = FocusPath::new(&["parent"]);
        let child_path = FocusPath::new(&["parent", "child"]);
        render_scroll_pane(
            &"Parent ancestor row 诊断\n".repeat(80),
            pane_block(
                "Parent / 父面板",
                self.focus.current() == Some(&parent_path),
                theme,
            ),
            area,
            buffer,
            &mut self.parent,
        );
        let mut regions = vec![region(&["parent"], area, area, 0, true, true)];
        if !self.child_rect.is_empty() {
            Clear.render(self.child_rect, buffer);
            render_scroll_pane(
                &"Nested child row 中文\n".repeat(60),
                pane_block(
                    "Nested / 子面板",
                    self.focus.current() == Some(&child_path),
                    theme,
                ),
                self.child_rect,
                buffer,
                &mut self.child,
            );
            regions.push(region(
                &["parent", "child"],
                self.child_rect,
                inner,
                1,
                true,
                true,
            ));
            // A visible non-scrollable decoration routes wheel to its scrollable ancestor.
            let label = Rect::new(inner.x, inner.y, inner.width.min(22), 1);
            Paragraph::new("ancestor wheel label").render(label, buffer);
            regions.push(region(&["parent", "label"], label, inner, 1, false, false));
        }
        if self.modal {
            let modal = Rect::new(
                inner.x + inner.width / 4,
                inner.y + inner.height / 4,
                inner.width / 2,
                inner.height / 2,
            )
            .intersection(inner);
            Clear.render(modal, buffer);
            Paragraph::new("Modal owns input\n背景输入被遮挡")
                .block(pane_block("Modal", true, theme))
                .render(modal, buffer);
            regions.push(region(&["modal"], modal, area, 10, true, false));
        }
        self.hits = HitMap::new(self.generation, regions);
        self.hits.reconcile_focus(&mut self.focus);
    }
}
pub fn mouse(column: u16, row: u16, kind: MouseKind) -> MouseEvent {
    MouseEvent {
        column,
        row,
        kind,
        modifiers: Modifiers::NONE,
    }
}
/// Runs one deterministic synthetic transition against a previously committed scene.
pub fn scenario(scene: &mut NestedScene, index: usize) -> &'static str {
    let child = scene.child_rect;
    match index {
        0 => {
            assert!(!child.is_empty(), "draw must precede the scenario");
            scene.pointer(mouse(
                child.x + 1,
                child.y + 1,
                MouseKind::Press(MouseButton::Primary),
            ));
            assert_eq!(
                scene.focus.current(),
                Some(&FocusPath::new(&["parent", "child"]))
            );
            "deepest-visible-hit"
        }
        1 => {
            let before = scene.parent.offset();
            scene.pointer(mouse(
                scene.parent_rect.x + 1,
                scene.parent_rect.y + 1,
                MouseKind::ScrollDown,
            ));
            assert!(scene.parent.offset() > before);
            assert_eq!(
                scene.focus.current(),
                Some(&FocusPath::new(&["parent", "child"]))
            );
            "ancestor-scroll-focus-preserved"
        }
        2 => {
            assert_eq!(
                scene.pointer(mouse(
                    scene.parent_rect.right(),
                    scene.parent_rect.bottom(),
                    MouseKind::ScrollDown
                )),
                PointerEffect::Consumed
            );
            "clip-outside-visible"
        }
        3 => {
            scene.set_modal(true);
            "open-modal"
        }
        4 => {
            let before = scene.parent.offset();
            assert_eq!(
                scene.pointer(mouse(
                    scene.parent_rect.x + 1,
                    scene.parent_rect.y + 1,
                    MouseKind::ScrollDown
                )),
                PointerEffect::Consumed
            );
            assert_eq!(scene.parent.offset(), before);
            "modal-background-blocked"
        }
        5 => {
            scene.set_modal(false);
            "close-modal"
        }
        6 => {
            let old = scene.hits.clone();
            scene.invalidate_geometry();
            assert_eq!(
                old.pointer(
                    mouse(child.x, child.y, MouseKind::ScrollDown),
                    scene.generation,
                    &mut scene.focus
                ),
                PointerEffect::StaleGeometry
            );
            "resize-invalidates-geometry"
        }
        _ => panic!("unknown bounded scenario"),
    }
}
