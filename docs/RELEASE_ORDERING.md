# Future publication ordering

The first public release of `jerry-terminal-ui` is intentionally outside the
HookStat v0.3 G20–G23 train.

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
