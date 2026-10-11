# terminal-ui-runtime (unreleased 0.1.0 candidate)

Single-flight read-only collection, one coalesced pending request, generation and
epoch rejection, immutable last-accepted snapshots and content revision.
Collectors must honor cooperative deadlines and never capture product writers.
`shutdown(budget)` returns an error and retains ownership when a worker fails to
stop. Drop is a last-resort join, not a bounded stop guarantee. Product adapters
own actual commands, stage budgets, child process groups and stdout limits.

MIT, Rust 1.97.1, Edition 2024. Depends on terminal-ui-contract =0.2.0.
No publication/real Provider/product integration is implied by local tests.
