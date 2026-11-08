# terminal-ui-contract Agent Instructions

## Workspace map

Read the [workspace README](README.md) ([简体中文](README.zh-CN.md)) for
dependency selection and example commands. The workspace has two interactive entry points.

- `crates/ratatui` (`terminal-ui-ratatui`) adapts Core to Ratatui/Crossterm
  for full-screen TUIs. Start with `examples/gallery.rs` or `examples/nested.rs` there.
- `crates/cliclack` (`terminal-ui-cliclack`) adapts Core to inline CLI
  prompts. Start with `PromptSession` in `src/lib.rs` and `examples/menu.rs`
  there. Consumers pin a Git revision.

The root crate (`terminal-ui-contract`, under `src/`) owns backend-neutral
UI semantics. `crates/runtime` (`terminal-ui-runtime`) provides optional
cooperative read-only refresh. See the [documentation index](docs/README.md)
for the source map and historical records.

## Boundaries

Keep Core's public API dependency-neutral. Do not expose Ratatui, Crossterm,
cliclack, HookStat, TabBeacon, or product-domain types. Add a semantic
primitive only when at least two consumers can use the same behavior.
Preserve deterministic behavior, CJK display-cell safety, press-only input,
and the terminal Human-interface contract.

Applications own domain validation and writes. Terminal presentation belongs
to one owner at a time. Keep help, machine-readable output and redirected paths
separate from interactive sessions.

Use workspace-wide validation when a change crosses crate boundaries. Do not
publish, tag, or release without separate explicit release authorization.
