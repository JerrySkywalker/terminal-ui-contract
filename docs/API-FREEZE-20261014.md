# TUC public API freeze record v1 — 2026-10-14

Decision: TUC-V020-EXACT-API-FREEZE-20261014. OWNER_API_FREEZE=APPROVED_FROZEN.
Owner approval date: 2026-10-14 (Asia/Shanghai); recorded UTC: 2026-10-14T04:23:26.352291+00:00.
Authority: the Owner's direct instruction and section 0 of Goal `TUC-V020-API-FREEZE-AND-PUBLICATION-ADMISSION-12H-001`.
This supersedes the current PENDING_OWNER status in earlier dated records;
their original observations and failures remain historical, unchanged.

Frozen API source: `eb16cd91f924c145180b5befcf9f2cacc7a46d84` in JerrySkywalker/terminal-ui-contract.
Accepted real consumer: TabBeacon `548f1f83b9873ec997436a6049b1dc3068eabeea` (Draft #123).
The three consumer Git/rev pins stay at the original frozen API source.
A later documentation commit does not move that approved API baseline.
Any public signature, permission default or behavior change requires a new Owner
decision; publication of a different final source/package requires exact review.

| Crate | Frozen version | Dependencies and exposure |
| --- | --- | --- |
| terminal-ui-contract | 0.2.0 | Dependency-neutral Core; unicode-segmentation 1.12, unicode-width 0.2, serde 1 with derive, toml 0.9. No backend/product types. |
| terminal-ui-runtime | 0.1.0 | Core =0.2.0; product-authoritative cooperative collectors and child lifecycle. |
| terminal-ui-ratatui | 0.1.0 | Core =0.2.0; Ratatui =0.30.2 with crossterm, Crossterm =0.29.0; Runtime =0.1.0 dev-only. Backend types belong here. |

All three are MIT, Edition 2024, Rust 1.97.1. Cargo.lock is the exact resolved
dependency graph; the manifest requirements above are not substitutes for it.

## Frozen capabilities and compatibility

Core's existing 0.1 chrome/footer/input/interaction/layout/locale/navigation/text
exports remain accessible. Added public modules are keymap, focus, viewport,
document and theme. Standard/Vim/Custom, Normal/Insert and Direct/Pane remain
independent choices. ActionRegistry and KeymapRegistry accept registered typed
actions and validated TOML; Resolver produces Resolution/ActionIntent, never
execution or permission. Help/footer use the effective context map. Capture
contexts stop fallback; modal writes are suppressed by default. Only explicit
registered product contexts may opt into typed ProductWriteRequest, with product
confirmation, ownership, allowlist and transaction checks still required.

The existing sequence default (600ms), accepted range (50..=5000ms), sequence
length (1..=4) and navigation count cap (9999) remain unchanged. These are source
API values, not a physical delivery or end-to-end latency promise. Source
ResponsivePolicy defaults remain 24x10 and 21+40 columns; compact/TooSmall
behavior is preserved. They do not promise simultaneous readable panes at 24
columns. TabBeacon's Standard/Pane selection and collector budgets are product
policy, not mandatory defaults for every consumer.

FocusState/FocusPath transitions produce FocusChanged only;
FOCUS_CHANGED_IMPLIES_PRODUCT_WRITE=false. HitMap uses committed geometry,
generation, ancestor clips and modal routing. Wheel targets the nearest scroll
ancestor without changing focus. BoundedViewport clamps display rows; grapheme
and CJK handling, Smart/Raw shared UiDocument facts, en-US/zh-CN and NO_COLOR
precedence remain as implemented. No new watcher, Plot/provider framework or
shared persistent writer is introduced.

Runtime's ReadOnlyCollector, RefreshController, RefreshRequest generation/epoch,
CollectionControl, ResourceState and immutable Snapshot retain their existing
signatures. Epoch/generation are u64 values, not new public wrapper types.
One worker, one active and at most one pending request remain owned. Deadlines
and cancellation are cooperative; shutdown(budget) bounds caller waiting and
retains JoinHandle on timeout. Drop Join may block without a bound. Strict
OS/native I/O stall termination is not guaranteed. Rust traits do not create
process or permission isolation; products own collector admission and actual
command/child/output budgets.

Ratatui retains input adaptation, TerminalGuard/TerminalOps, shared widgets,
Focus/HitMap/Viewport rendering and the Gallery/nested examples. The exported
synthetic module is part of this exact surface but proves only fixtures.
TerminalGuard records attempted entry and reverses cleanup, reports finish
failures and retries in Drop. Successful cleanup bytes do not prove DEC1049,
mouse or paste state without trusted readback; physical restoration remains
independent. Callers must own the terminal session.

PermittedText::from_product_redaction declares a product trust boundary; it
does not sanitize arbitrary strings. Products must redact payloads, commands,
secrets and sensitive paths before construction. Snapshot freshness never
creates provider truth. HookStat-labelled synthetic fixtures do not prove real
HookStat migration. Physical keys/mouse, Owner visual acceptance, optical
input-to-screen timing and real Provider/Hook qualification are not inherited.

TabBeacon bounded contract v1 is accepted at the exact consumer SHA above:
original Preview baseline, observed drift refusal, registered cooperating
conditional writers and limited fail-closed recovery. Arbitrary foreign-writer
CAS and cross-file crash atomicity are UNSUPPORTED_BY_V090_CONTRACT;
their original UNPROVEN/BLOCKED counterexamples are preserved, never PASS.

## Verification and publication boundary

The Goal delivery contains FROZEN-API-INVENTORY.md, compiler-rendered Rustdoc
signatures, source fingerprints and API-COMPATIBILITY-MATRIX.md. Source hashes
alone are not a mechanical public API proof. Coverage and target/cfg limitations
are explicit; no new global analysis tool is installed.

Public API freeze and exact implementation acceptance are already APPROVED.
They do not authorize merge, tag, publication/upload, Release, install,
Hook trust, daily configuration, real Provider/Hook/L4 or marking Draft Ready.
Public registry resolution, publisher rights, exact package acceptance and
consumer qualification remain separate gates. No authenticated Cargo operation
or credential access is part of this admission.
END_GATE=STOP_FOR_OWNER_PUBLICATION_AUTHORIZATION_REVIEW.
