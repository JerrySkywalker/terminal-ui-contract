Current candidate clarification: [v0.9.0 bounded policy and API status](V090-BOUNDED-POLICY.md). The historical rows below are dated source references, not current consumer pins. TabBeacon Draft #123 now uses Core =0.2.0 / Runtime =0.1.0 / Ratatui =0.1.0 at one exact Git revision; released consumers and real HookStat adoption remain separate. Public API freeze is PENDING_OWNER.

# Qualification and release boundary

The source candidate is reviewed through one Draft PR and one normal PR CI on
Windows/Linux with Rust 1.97.1. Development runs affected test families only.
CI checks out the exact PR head, checks fmt/clippy, tests/builds the workspace and
checks package assembly. A passing source CI does not authorize publication or
prove physical input/real provider currentness.

| Family | Source and focused evidence |
| --- | --- |
| INPUT_MODAL_KEYMAP | tests/keymap.rs; adapter input unit test; synthetic gallery writer/editor/overlay integration |
| FOCUS_VIEWPORT_GEOMETRY | tests/focus_viewport.rs; adapter tests/focus_viewport.rs; compact/gallery integration |
| ASYNC_RESOURCE_SNAPSHOT | runtime tests/refresh.rs and owned_child.rs; gallery slow input+frame fixture |
| SMART_RAW_SEMANTIC | tests/document_theme.rs; gallery projection/allowlist/NO_COLOR/locale tests |
| TWO_CONSUMER_CONFORMANCE | synthetic TabBeacon nine screens and HookStat Events/Handlers/Details/Window/List use the same Core/Runtime/Adapter APIs |
| PACKAGING_RELEASE | Cargo manifests, LICENSE, version matrix, workspace package preflight and final PR CI |

An ignored test function in owned_child.rs is a 60s child harness. Its parent
test explicitly launches that exact function, observes FIXTURE_READY output,
terminates only its Child handle and verifies reaping; ignored does not mean
the lifecycle behavior is omitted.

Runtime includes real 2s and 10s cooperative collectors plus controlled hang,
panic, stale generation/epoch, 100 unchanged responses and shutdown-retention
tests. The slow TestBackend fixture samples 100 local input/render cycles while
a 2s collector is active. This is synthetic responsiveness evidence; it is not
real Windows Terminal input-to-frame P95, nor an accepted <=100ms contract.

Redaction is product-authoritative: the synthetic adapter forwards an allowlist
and omits private payload/command fields. PermittedText is a declaration by that
adapter, not a generic secret detector. No raw sessions/credentials are collected.

## Version and dependency matrix

| Package | Candidate | Dependencies / boundary |
| --- | --- | --- |
| terminal-ui-contract | 0.2.0 | Unicode segmentation/width, serde/TOML; no backend/domain types |
| terminal-ui-runtime | 0.1.0 | Core =0.2.0; product-owned collectors and child protocol |
| terminal-ui-ratatui | 0.1.0 | Core =0.2.0, Ratatui =0.30.2, Crossterm =0.29.0; Runtime =0.1.0 dev-only |
| existing TabBeacon/HookStat | unchanged | continue to pin released Core =0.1.0; no product migration |

All workspace packages: MIT, Edition 2024, Rust 1.97.1. Cargo.lock pins the actual
dependency set; adapter exposes backend types only in its own package. No GPL
widget, Plot, CSIS dependency or unadopted provider scaffolding is introduced.
Ratatui/Crossterm are admitted from exact sealed consumer locks and tested locally;
third-party widgets are not needed for this first-wave implementation.

Read-only source references: TUC base 7fde0040f3135e2772461833846b5525732564ea;
HookStat MIT refresh/state at 9e0e082203ea4026bba9c755a1509a0e9cb03831;
TabBeacon PR123 7f098b7e770ed41b5000aabe15bef11f6013a7e1; coordinator
73391eb7c899b9beec6223596a9fd6ec0077e632. HookStat informed the
single-flight/last-good design; no product types or source files are copied.

## Unproven and Owner next gate

- Physical Windows Terminal mouse/key delivery, restoration and visual acceptance.
- Real input-to-frame P95 and any chosen performance limit/device/sample contract.
- Real Provider/Hook/ledger currentness, product business truth, writes/CAS/trust.
- Registry name reservation/ownership and externally resolving all new semvers
  after publication. HTTP 404 only shows absence at observation time.
- Main merge, tags, crates.io publish, GitHub Release, daily install and product migration.

Publication order when separately authorized: Core 0.2.0 -> Runtime 0.1.0 and
Ratatui 0.1.0 -> optional separately-authorized consumer integration. Package
assembly/pre-publication dry-run must not be presented as an uploaded registry
package. Detailed exact-head test/CI receipts and capability disposition live in
the Goal artifact root; final Owner review may freeze/change these candidate APIs.
