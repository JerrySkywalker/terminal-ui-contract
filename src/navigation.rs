//! One-current-screen top-level navigation primitive.

/// A generic current-screen state with no selected/active split.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TopLevelNavigation<Screen> {
    current: Screen,
}

impl<Screen: Copy + Eq> TopLevelNavigation<Screen> {
    #[must_use]
    pub const fn new(current: Screen) -> Self {
        Self { current }
    }

    #[must_use]
    pub const fn current(self) -> Screen {
        self.current
    }

    pub fn set_current(&mut self, screen: Screen) {
        self.current = screen;
    }

    /// Moves directly to the next/previous screen in a consumer-owned order.
    /// No intermediate focus or activation operation exists.
    #[must_use]
    pub fn move_by(&mut self, ordered: &[Screen], delta: isize) -> bool {
        let Some(index) = ordered.iter().position(|screen| *screen == self.current) else {
            return false;
        };
        if ordered.is_empty() {
            return false;
        }
        let next = (index as isize + delta).rem_euclid(ordered.len() as isize) as usize;
        self.current = ordered[next];
        true
    }
}

#[cfg(test)]
mod tests {
    use super::TopLevelNavigation;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Screen {
        Overview,
        Hooks,
        Settings,
    }

    #[test]
    fn direct_navigation_has_only_one_current_screen() {
        let mut nav = TopLevelNavigation::new(Screen::Overview);
        assert!(nav.move_by(&[Screen::Overview, Screen::Hooks, Screen::Settings], 1));
        assert_eq!(nav.current(), Screen::Hooks);
        assert!(nav.move_by(&[Screen::Overview, Screen::Hooks, Screen::Settings], -1));
        assert_eq!(nav.current(), Screen::Overview);
    }
}
