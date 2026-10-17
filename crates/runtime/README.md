> Current decision — 2026-10-14: Owner approved this exact API in TUC source `eb16cd91f924c145180b5befcf9f2cacc7a46d84` (TUC-V020-EXACT-API-FREEZE-20261014). Earlier candidate statements below are historical. Cooperative cancellation/Drop and terminal/redaction limitations remain; publication and real consumer/physical qualification are separate.

# terminal-ui-runtime (unreleased 0.1.0 candidate)

Single-flight read-only collection, one coalesced pending request, generation and
epoch rejection, immutable last-accepted snapshots and content revision.
Collectors must honor cooperative deadlines and never capture product writers.
`shutdown(budget)` returns an error and retains ownership when a worker fails to
stop. Drop is a last-resort join, not a bounded stop guarantee. Product adapters
own actual commands, stage budgets, child process groups and stdout limits.

MIT, Rust 1.97.1, Edition 2024. Depends on terminal-ui-contract =0.2.0.
No publication/real Provider/product integration is implied by local tests.

Owner-approved TabBeacon v0.9.0 bounded policy does not freeze this public API.
Only admitted cooperative collectors are covered. Strict native/OS I/O stall
exit is unguaranteed, and Drop Join remains unbounded. No thread model or
permission default changes; public API acceptance and publication remain Owner gates.
