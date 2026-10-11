# Explicit keymap loading

The product chooses the file path and passes UTF-8 text to
`KeymapRegistry::from_toml`. No cwd scan, remote load, implicit persistence,
backup/CAS operation or live watcher exists. Invalid overlays never mutate an
accepted registry. The product selects a safe base/last accepted fallback and
surfaces the returned validation error. Revert-to-default reconstructs the base.

```toml
schema_version = 1
preset = "vim"
navigation = "pane"
sequence_timeout_ms = 600

[[unbind]]
context = "global"
keys = ["F2"]

[[bindings]]
context = "global"
mode = "normal"
keys = ["F3"]
action = "ui.view.toggle_density"
```

Only registered typed IDs are accepted. For example a product must register
`tabbeacon.diagnostics.next_mode` before binding it in `diagnostics`; an arbitrary
`command`, `script` or unknown `shell.exec` is invalid. A key that requests a
registered product write still grants no permission, bypasses no confirmation
and multiplies no writes via a Vim count.

`overlay_toml` can only declare the base preset/navigation; initial loading can
select them. Insert and recovery keys are protected. `diagnostics(context,
required_actions)` reports allowed context shadows and unreachable actions;
`effective_hints(context)` includes only reachable bindings for current ownership.

Offline validation example (explicit file path only):

```text
cargo run -p terminal-ui-contract --example keymap-check -- path/to/keymap.toml
```

The example loads no product actions. A product should pass its own declarations
and contexts to the same API; no new standalone service or persistence layer is
needed. Physical key availability remains terminal-specific.
