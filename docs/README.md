# Terminal UI Contract documentation

Start with the [workspace README](../README.md) or [简体中文](../README.zh-CN.md)
to choose between the Ratatui full-screen adapter and the cliclack inline adapter.

## Source entry points

| Need | Package entry point | First example or reference |
| --- | --- | --- |
| Full-screen TUI | [Ratatui README](../crates/ratatui/README.md), [module exports](../crates/ratatui/src/lib.rs) | [Gallery](../crates/ratatui/examples/gallery.rs), [nested panes](../crates/ratatui/examples/nested.rs) |
| Inline CLI prompts | [cliclack README](../crates/cliclack/README.md), [`PromptSession`](../crates/cliclack/src/lib.rs) | [Menu](../crates/cliclack/examples/menu.rs), [wizard](../crates/cliclack/examples/wizard.rs) |
| Shared UI semantics | [Core module exports](../src/lib.rs) | [Keymap loading](KEYMAP.md), [keymap-check](../examples/keymap-check.rs) |
| Read-only refresh | [Runtime README](../crates/runtime/README.md), [`RefreshController`](../crates/runtime/src/refresh.rs) | [Refresh tests](../crates/runtime/tests/refresh.rs) |

Applications own terminal session boundaries, domain validation, writes and
redaction. Terminal examples and synthetic tests provide distinct evidence.
See each adapter's README for its terminal requirements.

## Distribution and history

- [Publication ordering](RELEASE_ORDERING.md) describes how to qualify a
  registry release and its consumers.
- [0.2 API design](API-DESIGN.md) explains the original Core, Runtime and
  Ratatui design. [The October 14 API freeze](API-FREEZE-20261014.md) records
  the accepted source and the scope of that decision.
- [Qualification record](QUALIFICATION.md) and [TabBeacon v0.9.0 policy](V090-BOUNDED-POLICY.md)
  preserve dated evidence and its limitations. Use the workspace README and
  Cargo manifests for current crate selection.
