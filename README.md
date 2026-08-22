# jerry-terminal-ui

`jerry-terminal-ui` is the dependency-neutral Human-interface contract shared
by JerrySkywalker terminal applications. It owns semantic shell, navigation,
footer, edit, overlay, input, locale, and display-width behavior. Product
adapters own their concrete Ratatui/Crossterm rendering and domain content.

The crate is intentionally not published during its initial adoption train.
Consumers use a pinned Git revision until a separately approved release train
publishes the crate first, then updates and publishes consumers in dependency
order.

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

CI covers Windows and Linux. No crates.io publication, release tag, or GitHub
Release is part of this repository's bootstrap or the HookStat v0.3 train.
