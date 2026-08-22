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
