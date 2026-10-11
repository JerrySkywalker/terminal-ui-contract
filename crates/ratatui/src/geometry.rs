//! Frame-specific hit regions from actual Ratatui rectangles.
use ratatui::layout::{Position, Rect};
use terminal_ui_contract::{
    focus::{FocusChanged, FocusPath, FocusState},
    input::{MouseButton, MouseEvent, MouseKind},
};

#[derive(Clone, Debug)]
pub struct HitRegion {
    pub path: FocusPath,
    pub rect: Rect,
    pub clip: Rect,
    pub z: u16,
    pub enabled: bool,
    pub focusable: bool,
    pub scrollable: bool,
}

#[derive(Clone, Debug)]
pub struct HitMap {
    generation: u64,
    regions: Vec<HitRegion>,
}
impl HitMap {
    pub fn new(generation: u64, regions: Vec<HitRegion>) -> Self {
        Self {
            generation,
            regions,
        }
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    fn visible_rect(&self, region: &HitRegion) -> Rect {
        let mut rect = region.rect.intersection(region.clip);
        for ancestor in self
            .regions
            .iter()
            .filter(|r| region.path.is_within(&r.path) && region.path != r.path)
        {
            if !ancestor.enabled {
                return Rect::default();
            }
            rect = rect.intersection(ancestor.rect).intersection(ancestor.clip);
        }
        rect
    }
    pub fn reconcile_focus(&self, focus: &mut FocusState) -> Option<FocusChanged> {
        focus.reconcile(
            self.regions
                .iter()
                .filter(|r| r.enabled && r.focusable && !self.visible_rect(r).is_empty())
                .map(|r| r.path.clone())
                .collect(),
        )
    }
    fn hit(&self, column: u16, row: u16, focus: &FocusState) -> Option<&HitRegion> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.enabled
                    && focus.allows(&r.path)
                    && self.visible_rect(r).contains(Position::new(column, row))
            })
            .max_by_key(|(index, r)| (r.z, r.path.parts().len(), *index))
            .map(|(_, r)| r)
    }
    fn ancestor(
        &self,
        region: &HitRegion,
        focus: &FocusState,
        predicate: impl Fn(&HitRegion) -> bool,
    ) -> Option<FocusPath> {
        self.regions
            .iter()
            .filter(|r| {
                r.enabled
                    && focus.allows(&r.path)
                    && region.path.is_within(&r.path)
                    && !self.visible_rect(r).is_empty()
                    && predicate(r)
            })
            .max_by_key(|r| r.path.parts().len())
            .map(|r| r.path.clone())
    }
    /// First click returns only FocusChanged. No ActionIntent can be emitted.
    /// A stale frame generation is consumed; callers must relayout after resize.
    pub fn pointer(
        &self,
        event: MouseEvent,
        expected_generation: u64,
        focus: &mut FocusState,
    ) -> PointerEffect {
        if expected_generation != self.generation {
            return PointerEffect::StaleGeometry;
        }
        let Some(region) = self.hit(event.column, event.row, focus) else {
            return PointerEffect::Consumed;
        };
        match event.kind {
            MouseKind::Press(MouseButton::Primary) => {
                let Some(path) = self.ancestor(region, focus, |r| r.focusable) else {
                    return PointerEffect::Consumed;
                };
                focus
                    .focus(&path)
                    .map_or(PointerEffect::Consumed, PointerEffect::Focus)
            }
            MouseKind::ScrollUp | MouseKind::ScrollDown => self
                .ancestor(region, focus, |r| r.scrollable)
                .map_or(PointerEffect::Consumed, |path| PointerEffect::Scroll {
                    path,
                    lines: if event.kind == MouseKind::ScrollUp {
                        -3
                    } else {
                        3
                    },
                }),
            _ => PointerEffect::Consumed,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointerEffect {
    Focus(FocusChanged),
    Scroll { path: FocusPath, lines: isize },
    Consumed,
    StaleGeometry,
}
