> Current decision — 2026-10-14: [exact public API freeze](API-FREEZE-20261014.md) is Owner-approved at `eb16cd91f924c145180b5befcf9f2cacc7a46d84` for Core 0.2.0 / Runtime 0.1.0 / Ratatui 0.1.0. Earlier candidate/PENDING_OWNER statements below are preserved historical context. Publication, physical qualification and installation remain separate gates. No implementation or consumer pin changes.

# Publication ordering

The first public release of `terminal-ui-contract` precedes consumer releases.

Before a consumer can publish a crates.io version that names this crate:

1. settle and release the exact shared-crate revision to crates.io;
2. wait for registry availability and verify an external consumer can resolve
   that crate version;
3. replace each development Git pin with the released semver dependency;
4. run the consumer's exact-head package and install checks; and
5. publish consumers only after their source resolves the released shared
   crate version.

The Git revision pins used during development are reproducible integration
inputs, not publishable crates.io dependency declarations.
