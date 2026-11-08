# terminal-ui-runtime

Optional cooperative read-only refresh for terminal applications. A
`RefreshController` owns one active collection and at most one coalesced
pending request. Generation and epoch checks reject stale results. The
controller preserves the last accepted immutable snapshot and tracks
content revision separately from sampling metadata.

`terminal-ui-runtime` 0.1.0 is available on [crates.io](https://crates.io/crates/terminal-ui-runtime/0.1.0).
It uses Core =0.2.0. Rust 1.97.1, Edition 2024, MIT.

Start with [`RefreshController` and `ReadOnlyCollector`](src/refresh.rs)
and the [refresh tests](tests/refresh.rs). The
[workspace README](https://github.com/JerrySkywalker/terminal-ui-contract)
covers Core and both interactive adapters.

Collectors must cooperate with deadlines and cancellation. The
application owns collector admission, commands, child process groups,
stage budgets and output limits. Keep writers outside the collector.
`shutdown(budget)` reports timeout and retains the worker handle so
the caller can retry. Drop joins the worker and may block indefinitely
if the collector fails to cooperate.

See the [historical API freeze](https://github.com/JerrySkywalker/terminal-ui-contract/blob/main/docs/API-FREEZE-20261014.md)
for the accepted scope and qualification limits.
