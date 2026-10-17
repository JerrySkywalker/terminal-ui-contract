# Changelog

## 0.1.0 candidate (unreleased)

Single-flight cooperative read-only collection, immutable snapshots, generation
and epoch acceptance, last-good freshness and owned-child lifecycle protocol.
Requires terminal-ui-contract =0.2.0 and Rust 1.97.1.
Shutdown timeout retains worker ownership; Drop can block for a collector that
violates cooperation. No strict OS-stall exit, product authority or publication
is implied. Public API freeze remains an Owner decision.
