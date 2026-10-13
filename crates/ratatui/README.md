# terminal-ui-ratatui (unreleased 0.1.0 candidate)

Neutral input adaptation, frame Rect hit-testing, Unicode bounded ScrollPane,
responsive shell, paired terminal lifecycle and thin Ratatui components.
The synthetic TabBeacon/HookStat Gallery runs only a TestBackend; it does not
execute Providers, Hooks, product writers or actual terminal input.

MIT, Rust 1.97.1, Edition 2024. Core =0.2.0, Ratatui =0.30.2, Crossterm =0.29.0.
Runtime =0.1.0 is a dev dependency for synthetic qualification. Actual terminal
key/mouse/visual evidence and product permissions require separate Owner gates.
## Visible nested example

`cargo run -p terminal-ui-ratatui --example nested` runs seven deterministic
synthetic transitions on TestBackend. Add `-- --real --auto` to render the same
scene and assertions on a real Crossterm backend and exit automatically. Use
`-- --real` for later attended inspection (`q` quits, `m` toggles the modal,
Tab cycles focus, click focuses, hover wheel scrolls).

The parent/child panes use shared `pane_block`, `render_scroll_pane`,
`BoundedViewport`, `FocusState` and `HitMap`. Hit regions are derived from the
rectangles actually drawn. The scene covers deepest visible hit, decoration
to ancestor scroll, clipping, modal background consumption and geometry
invalidation on resize. `tests/visible_nested.rs` compiles the same example
scene; it also verifies child visibility and focus reconciliation at small size.
Helpers remain example/test-only; Core/Runtime API and scheduling are unchanged.

Automatic events are synthetic. A successful backend draw proves software
submission, not a physical key-to-screen endpoint or human acceptance. Parent
recovery after forced process termination is a separate harness responsibility.

Owner-approved TabBeacon v0.9.0 bounded policy does not freeze this public API.
TerminalGuard cleanup output does not prove DEC1049/mouse/paste restoration;
without trusted DECRQM those states remain UNPROVEN. Products must redact before
PermittedText construction. Synthetic HookStat is not real product migration,
and software submission is not optical latency or human/visual qualification.
