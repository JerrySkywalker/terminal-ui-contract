# terminal-ui-contract

`terminal-ui-contract` is the dependency-neutral Human-interface contract shared
by JerrySkywalker terminal applications. It owns semantic shell, navigation,
footer, edit, overlay, input, locale, and display-width behavior. Product
adapters own their concrete Ratatui/Crossterm rendering and domain content.

The crate is published on crates.io as `terminal-ui-contract` 0.1.0. Consumers
bind the exact released semver version before publishing in dependency order.

## Scope

- no product-specific domain concepts;
- no terminal backend dependencies;
- deterministic state-machine tests;
- en-US and zh-CN locale-selection primitives;
- display-cell-safe CJK truncation.

## Development

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --locked
```

CI covers Windows and Linux. Releases require explicit authorization and must
bind the tag, GitHub Release, and crates.io package to the same accepted commit.

## v0.2 workspace candidate

The unreleased candidate adds neutral input/keymap/focus/viewport/documents to
Core and two separate packages: `terminal-ui-runtime` 0.1.0 (bounded read-only
refresh/snapshots) and `terminal-ui-ratatui` 0.1.0 (backend adapter, components and
synthetic Gallery). Core retains all 0.1 public APIs and has no Ratatui/Crossterm
dependency. Rust 1.97.1, Edition 2024, MIT apply to all three packages.

```text
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p terminal-ui-ratatui --example gallery -- --consumer tabbeacon --size 100x32 --no-color
cargo run -p terminal-ui-ratatui --example gallery -- --consumer hookstat --locale zh-CN --raw --size 40x16 --no-color
cargo run -p terminal-ui-ratatui --example gallery -- --consumer tabbeacon --components --no-color
```

Gallery exports TestBackend text frames only: no real terminal, user configuration,
Provider, ledger or Hook. See [API design](docs/API-DESIGN.md),
[keymap](docs/KEYMAP.md) and [qualification](docs/QUALIFICATION.md).
Actual terminal mouse/keyboard delivery and key-to-frame latency require separate
Owner qualification. Merge and publication are separate authorizations.
