# terminal-ui-ratatui (unreleased 0.1.0 candidate)

Neutral input adaptation, frame Rect hit-testing, Unicode bounded ScrollPane,
responsive shell, paired terminal lifecycle and thin Ratatui components.
The synthetic TabBeacon/HookStat Gallery runs only a TestBackend; it does not
execute Providers, Hooks, product writers or actual terminal input.

MIT, Rust 1.97.1, Edition 2024. Core =0.2.0, Ratatui =0.30.2, Crossterm =0.29.0.
Runtime =0.1.0 is a dev dependency for synthetic qualification. Actual terminal
key/mouse/visual evidence and product permissions require separate Owner gates.
