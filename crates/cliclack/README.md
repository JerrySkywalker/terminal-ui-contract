# terminal-ui-cliclack

Direct Rust inline Select, MultiSelect, Input, Confirm and bounded progress adapter
using cliclack 0.5.6. Requires Rust 1.97.1, matching the workspace. Core
exports provide shared locale, color and display-cell behavior. Workspace version
0.1.0 is Git-only (`publish = false`). See the
[workspace README](../../README.md) for the Git pin and the separate Ratatui
adapter for full-screen TUIs.

`PromptSession` owns terminal presentation and restores the default cliclack theme,
prior stdout/stderr color policy and cursor on drop (including unwinding). stdin and
stderr must be terminals; TERM=dumb is refused before output. NO_COLOR overrides
color. Session contention fails immediately. Callers must not use another prompt
library/global theme concurrently. cliclack owns transient input mode, in-place
redraw and Unicode editing; this library does not install a process signal handler.
OS termination/abort cannot run Rust destructors. Input/output errors remain errors;
Esc and Ctrl+C become `PromptOutcome::Cancelled`, never process exit or submission.

`Choice` uses stable IDs; duplicates, empty enabled sets and invalid defaults are
rejected. Disabled items are omitted and cannot be selected. Labels/descriptions
remove control characters and truncate by Core grapheme/display-cell primitives;
visible rows adapt to terminal height. Domain text belongs to each product; basic
cancellation supports Core en-US/zh-CN. Prompts are drafts: the consumer validates
and performs any action separately. No focus/viewport/keymap engine, product types,
network operations or user configuration writes are part of this library.

Examples: `cargo run -p terminal-ui-cliclack --example menu` or `--example wizard`.
Run them from an owned real terminal. Redirected, JSON and help paths must never
create a session. Attended checks cover 30 arrow presses, Esc/Ctrl+C, cursor
and echo, CJK, NO_COLOR, 50/70/100 columns on Windows Terminal and WSL zsh.
