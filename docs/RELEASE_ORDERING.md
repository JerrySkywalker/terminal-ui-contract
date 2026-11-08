# Publication ordering

See the [workspace README](../README.md) for currently available versions.
A new release, tag or registry publication requires separate explicit
authorization. Core, Runtime and Ratatui have registry packages. The
cliclack adapter has `publish = false` and is consumed by Git revision.

For a future release of the shared registry packages:

1. Settle the exact source and version of every package in scope.
2. Publish Core before dependents that require that version. Publish Runtime
   before Ratatui when Ratatui's package verification requires the new
   Runtime development dependency.
3. Verify the released versions resolve in an external registry consumer.
4. Bind the consumer to the released versions and run its package and
   install checks against that revision before consumer publication.

A Git revision pin is suitable for a Cargo build from source. A Git-only
normal dependency such as `terminal-ui-cliclack` needs a separate
distribution decision before its consumer can publish to crates.io.

See the [historical API freeze](API-FREEZE-20261014.md) for the scope of
the October 2026 Core, Runtime and Ratatui decision.
