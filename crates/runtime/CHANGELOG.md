# Changelog

## 0.1.0

Single-flight cooperative read-only collection, immutable snapshots, generation
and epoch acceptance, last-good freshness and owned-child lifecycle protocol.
Requires terminal-ui-contract =0.2.0 and Rust 1.97.1.
Shutdown timeout retains worker ownership; Drop can block for a collector that
violates cooperation. Strict OS-stall exit and product authority are outside
the runtime contract.
