//! Stable Human application-shell geometry.

/// The normative shell used by the shared Human interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellGeometry {
    pub header_rows: u16,
    pub footer_rows: u16,
    pub sidebar_columns: u16,
    pub minimum_width: u16,
    pub minimum_height: u16,
}

/// Header 2, footer 2, sidebar 21, minimum 24x10.
pub const HUMAN_SHELL: ShellGeometry = ShellGeometry {
    header_rows: 2,
    footer_rows: 2,
    sidebar_columns: 21,
    minimum_width: 24,
    minimum_height: 10,
};

impl ShellGeometry {
    #[must_use]
    pub const fn supports(self, width: u16, height: u16) -> bool {
        width >= self.minimum_width && height >= self.minimum_height
    }
}

#[cfg(test)]
mod tests {
    use super::HUMAN_SHELL;

    #[test]
    fn shell_contract_is_stable_at_minimum_and_normal_sizes() {
        assert!(HUMAN_SHELL.supports(24, 10));
        assert!(!HUMAN_SHELL.supports(23, 10));
        assert!(!HUMAN_SHELL.supports(24, 9));
        assert_eq!(HUMAN_SHELL.header_rows, 2);
        assert_eq!(HUMAN_SHELL.footer_rows, 2);
        assert_eq!(HUMAN_SHELL.sidebar_columns, 21);
    }
}
