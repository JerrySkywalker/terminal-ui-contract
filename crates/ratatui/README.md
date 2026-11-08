# terminal-ui-ratatui

The full-screen TUI adapter for Terminal UI Contract. It translates Crossterm
input into Core events, pairs terminal entry and cleanup with `TerminalGuard`,
and provides responsive shell layout, focus and hit geometry, bounded Unicode
viewports and reusable Ratatui components.

`terminal-ui-ratatui` 0.1.0 is available on [crates.io](https://crates.io/crates/terminal-ui-ratatui/0.1.0).
It uses Core =0.2.0, Ratatui =0.30.2 and Crossterm =0.29.0. Runtime =0.1.0
is a development dependency. Rust 1.97.1, Edition 2024, MIT.

See the [workspace README](https://github.com/JerrySkywalker/terminal-ui-contract)
for dependency setup and the separate cliclack adapter for inline CLI prompts.

## Examples

```sh
cargo run --locked -p terminal-ui-ratatui --example gallery -- --consumer tabbeacon --size 100x32 --no-color
cargo run --locked -p terminal-ui-ratatui --example gallery -- --consumer hookstat --locale zh-CN --raw --size 40x16 --no-color
cargo run --locked -p terminal-ui-ratatui --example nested
```

Gallery and default `nested` use TestBackend. Gallery prints a text frame
from synthetic TabBeacon/HookStat data. `nested` exercises synthetic
transitions for hit testing, clipping, focus, modal routing and resize.

Run `cargo run --locked -p terminal-ui-ratatui --example nested -- --real`
in an owned terminal for attended inspection. Press `q` to quit, `m` to
toggle the modal, and Tab to cycle focus. Click to focus and use the wheel
over a pane to scroll it. Add `--auto` after `--real` for synthetic events
on the real backend with automatic exit.

## Integration boundaries

Own the terminal before entering `TerminalGuard`. Cleanup output and
backend draw completion establish software behavior. Physical keyboard,
mouse, visual, latency and terminal-mode restoration each require their own
evidence. Applications redact before constructing Core's `PermittedText`.

See the [historical API freeze](https://github.com/JerrySkywalker/terminal-ui-contract/blob/main/docs/API-FREEZE-20261014.md)
for the accepted scope and qualification limits.
