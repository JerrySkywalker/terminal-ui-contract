# jerry-terminal-ui Agent Instructions

This crate is shared Human-interface infrastructure. Keep its public API
dependency-neutral: do not expose Ratatui, Crossterm, HookStat, TabBeacon, or
product-domain types. Add a semantic primitive only when at least two
consumers can use the same behavior.

Preserve deterministic behavior, CJK display-cell safety, press-only input,
and the terminal Human-interface contract. Do not publish, tag, or release
without a separate explicit release authorization.
