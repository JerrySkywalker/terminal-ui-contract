//! Display-row viewport, reflow and responsive policy; backend computes rendered rows.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BoundedViewport {
    offset: usize,
    content_rows: usize,
    visible_rows: usize,
}
impl BoundedViewport {
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn content_rows(&self) -> usize {
        self.content_rows
    }
    pub fn visible_rows(&self) -> usize {
        self.visible_rows
    }
    pub fn max_offset(&self) -> usize {
        self.content_rows.saturating_sub(self.visible_rows)
    }
    pub fn reflow(&mut self, content_rows: usize, visible_rows: usize) {
        self.content_rows = content_rows;
        self.visible_rows = visible_rows;
        self.offset = self.offset.min(self.max_offset());
    }
    pub fn home(&mut self) {
        self.offset = 0;
    }
    pub fn end(&mut self) {
        self.offset = self.max_offset();
    }
    pub fn lines(&mut self, delta: isize) {
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
    }
    pub fn page(&mut self, delta: isize) {
        self.step(self.visible_rows.max(1), delta);
    }
    pub fn half_page(&mut self, delta: isize) {
        self.step((self.visible_rows / 2).max(1), delta);
    }
    pub fn ensure_visible(&mut self, row: usize) {
        let row = row.min(self.content_rows.saturating_sub(1));
        if row < self.offset {
            self.offset = row;
        } else if row >= self.offset.saturating_add(self.visible_rows) {
            self.offset = row.saturating_add(1).saturating_sub(self.visible_rows);
        }
        self.offset = self.offset.min(self.max_offset());
    }
    fn step(&mut self, rows: usize, delta: isize) {
        let distance = rows.saturating_mul(delta.unsigned_abs());
        self.offset = if delta < 0 {
            self.offset.saturating_sub(distance)
        } else {
            self.offset.saturating_add(distance)
        }
        .min(self.max_offset());
    }
    pub fn position_label(&self) -> String {
        if self.visible_rows == 0 {
            format!("0 / {}", self.content_rows)
        } else if self.content_rows == 0 {
            "0 / 0".into()
        } else {
            format!(
                "{}–{} / {}",
                self.offset.saturating_add(1),
                self.offset
                    .saturating_add(self.visible_rows)
                    .min(self.content_rows),
                self.content_rows
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutClass {
    Full,
    Compact,
    TooSmall,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponsivePolicy {
    pub minimum_width: u16,
    pub minimum_height: u16,
    pub navigation_width: u16,
    pub detail_minimum: u16,
}
impl Default for ResponsivePolicy {
    fn default() -> Self {
        Self {
            minimum_width: 24,
            minimum_height: 10,
            navigation_width: 21,
            detail_minimum: 40,
        }
    }
}
impl ResponsivePolicy {
    pub fn classify(self, width: u16, height: u16) -> LayoutClass {
        if width < self.minimum_width || height < self.minimum_height {
            LayoutClass::TooSmall
        } else if width < self.navigation_width.saturating_add(self.detail_minimum) {
            LayoutClass::Compact
        } else {
            LayoutClass::Full
        }
    }
}
