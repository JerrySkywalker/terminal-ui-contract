//! Independent pane focus. Every transition is view-only and has no action callback.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FocusPath(Vec<String>);

impl FocusPath {
    pub fn new(parts: &[&str]) -> Self {
        Self(parts.iter().map(|part| (*part).into()).collect())
    }
    pub fn parts(&self) -> &[String] {
        &self.0
    }
    pub fn is_within(&self, ancestor: &Self) -> bool {
        self.0.starts_with(&ancestor.0)
    }
    pub fn parent(&self) -> Option<Self> {
        if self.0.len() > 1 {
            Some(Self(self.0[..self.0.len() - 1].to_vec()))
        } else {
            None
        }
    }
    pub fn label(&self) -> String {
        self.0.join(" / ")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusChanged {
    pub previous: Option<FocusPath>,
    pub current: Option<FocusPath>,
}

/// A compile-time separation of focus effects from ActionIntent/product writers.
pub const FOCUS_CHANGED_IMPLIES_PRODUCT_WRITE: bool = false;

#[derive(Clone, Debug)]
struct Trap {
    root: FocusPath,
    restore: Option<FocusPath>,
}

#[derive(Clone, Debug, Default)]
pub struct FocusState {
    current: Option<FocusPath>,
    visible: Vec<FocusPath>,
    traps: Vec<Trap>,
    revision: u64,
}

impl FocusState {
    pub fn current(&self) -> Option<&FocusPath> {
        self.current.as_ref()
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn modal_root(&self) -> Option<&FocusPath> {
        self.traps.last().map(|t| &t.root)
    }
    pub fn allows(&self, path: &FocusPath) -> bool {
        self.modal_root().is_none_or(|root| path.is_within(root))
    }
    pub fn visible(&self) -> &[FocusPath] {
        &self.visible
    }

    /// Adapter passes focusable paths from the current frame. Resize/filter never
    /// leaves an invisible focus; prefer closest visible ancestor, then first pane.
    pub fn reconcile(&mut self, visible: Vec<FocusPath>) -> Option<FocusChanged> {
        self.visible = visible;
        let mut candidate = self.current.clone();
        while let Some(path) = &candidate {
            if self.visible.contains(path) && self.allows(path) {
                return self.set(candidate);
            }
            candidate = path.parent();
        }
        // An open modal with no visible children retains ownership (no background fallback).
        candidate = self.visible.iter().find(|path| self.allows(path)).cloned();
        self.set(candidate)
    }

    pub fn focus(&mut self, path: &FocusPath) -> Option<FocusChanged> {
        if !self.visible.contains(path) || !self.allows(path) {
            return None;
        }
        self.set(Some(path.clone()))
    }

    pub fn cycle(&mut self, backwards: bool) -> Option<FocusChanged> {
        let candidates: Vec<_> = self
            .visible
            .iter()
            .filter(|path| self.allows(path))
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let index = self
            .current
            .as_ref()
            .and_then(|p| candidates.iter().position(|v| v == p));
        let next = match index {
            Some(i) if backwards => (i + candidates.len() - 1) % candidates.len(),
            Some(i) => (i + 1) % candidates.len(),
            None => 0,
        };
        self.set(Some(candidates[next].clone()))
    }

    pub fn enter_child(&mut self) -> Option<FocusChanged> {
        let current = self.current.as_ref()?;
        let child = self
            .visible
            .iter()
            .find(|p| p.is_within(current) && p != &current && self.allows(p))
            .cloned()?;
        self.set(Some(child))
    }
    pub fn leave_child(&mut self) -> Option<FocusChanged> {
        let parent = self.current.as_ref()?.parent()?;
        self.focus(&parent)
    }

    /// Product opens a modal explicitly, before laying it out. It traps input
    /// immediately even when its frame has not been committed yet.
    pub fn push_modal(&mut self, root: FocusPath) -> Option<FocusChanged> {
        self.traps.push(Trap {
            root: root.clone(),
            restore: self.current.clone(),
        });
        self.set(Some(root))
    }
    pub fn pop_modal(&mut self) -> Option<FocusChanged> {
        let trap = self.traps.pop()?;
        self.set(trap.restore);
        let visible = self.visible.clone();
        let repaired = self.reconcile(visible);
        repaired.or_else(|| {
            Some(FocusChanged {
                previous: Some(trap.root),
                current: self.current.clone(),
            })
        })
    }
    fn set(&mut self, current: Option<FocusPath>) -> Option<FocusChanged> {
        if self.current == current {
            return None;
        }
        let previous = std::mem::replace(&mut self.current, current.clone());
        self.revision = self.revision.wrapping_add(1);
        Some(FocusChanged { previous, current })
    }
}

/// Stable selected identity is independent from focus and current screen.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StableSelection<Id> {
    selected: Option<Id>,
    index: usize,
}
impl<Id: Clone + Eq> StableSelection<Id> {
    pub fn selected(&self) -> Option<&Id> {
        self.selected.as_ref()
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn select(&mut self, ids: &[Id], index: usize) {
        self.index = index.min(ids.len().saturating_sub(1));
        self.selected = ids.get(self.index).cloned();
    }
    pub fn reconcile(&mut self, ids: &[Id]) {
        let index = self
            .selected
            .as_ref()
            .and_then(|id| ids.iter().position(|v| v == id))
            .unwrap_or(self.index);
        self.select(ids, index);
    }
}
