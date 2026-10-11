# v0.2 candidate API

Candidate for Owner review; this document does not freeze consumer defaults or
authorize merge, publication, real Providers, installation or trust.

The root `terminal-ui-contract` 0.2.0 is backend-neutral. Its original chrome,
footer, input, interaction, layout, locale, navigation and text exports remain
available, including `InputKind`, `admits_input`, `TopLevelNavigation`,
`SettingsEditor`, `OverlayState`, `FooterState::hints` and `HUMAN_SHELL`.
`KeymapProfile`, `InputMode` and `NavigationPolicy` are independent; choose
Standard/Pane for the synthetic TabBeacon fixture and opt into Vim. Existing
Direct navigation is preserved explicitly.

## Keymap and text ownership

`ActionRegistry` registers namespaced product identities with an effect and
allowed contexts. `KeymapRegistry` creates one immutable effective map from
built-ins, product bindings and a validated TOML overlay. `Resolver` receives a
monotonic clock and returns `Resolution`, never an executable or callback.
Both Help and footer use `effective_hints` from that same registry/context.

Contexts are supplied deepest owner first, current page then global. Editor,
Overlay and Confirm capture stop all lower fallback. Esc and Ctrl+C are reserved
cancel/quit **requests**; the product must still use its dirty/confirmation gate.
Insert receives literal characters/paste and separate editor keys. Unsupported
enhanced events remain explicit; terminals without disambiguation cannot promise
Ctrl+digit delivery. Modifiers are never reduced to a plain digit.

Sequences allow 1–4 chords, default timeout 600ms (50–5000ms accepted). Counts
cap at 9999 and apply only to navigation. Context/mode/focus changes and explicit
cancel/expiry clear state. The first matching owner owns the whole sequence;
common-first-chord shadows are diagnosed and lower sequences are absent from
hints. Same-owner duplicates/prefix ambiguity and unknown actions are errors.
Vim digits are counts; the synthetic product uses Alt+digits for query selection.
No Visual/Ex/macros, live watcher or key recorder is implemented.

## Focus and geometry

`TopLevelNavigation` is the single current screen. Independent `FocusState`
holds a `FocusPath`, visible focus order and nested modal traps. Every transition
returns only `FocusChanged`. `FOCUS_CHANGED_IMPLIES_PRODUCT_WRITE=false` is a
constant and the dual-consumer fixture exercises a functioning fake writer as
positive control: focus, hover, wheel, modal and density events cannot reach it.
Actual activation and permission/CAS/confirmation remain product-owned.

The adapter's `HitMap` uses committed frame Rects, visible ancestor clips,
absolute z/depth/draw order and geometry generation. Decorative children fall
back to a visible focusable/scrollable ancestor. Hover does not change focus.
Wheel targets the hit pane's nearest scrollable ancestor; no-hit and unrelated
non-scrollable panes consume without global navigation. Modal subtrees intercept
all pointer/keyboard input, even outside their body. Stale geometry is rejected.

`BoundedViewport` clamps by content display rows and visible rows on every
reflow. Core grapheme/cell reflow yields the exact rows rendered by the adapter,
without a second wrap algorithm or u16 scroll offset. Tabs become four spaces;
a grapheme wider than a one-column body becomes a visible `?`, never a split
cluster. Locale/filter/snapshot/density/resize changes must relayout/reflow.
`StableSelection` restores identity when the product's list changes.

Responsive candidate: minimum 24x10; Full when width >= navigation width +
minimum detail width (defaults 21 + 40), Compact one pane otherwise. Tab/Enter/Esc
in the fixture can expose hidden navigation/details. TooSmall draws identity,
resize guidance and recovery. These breakpoints are candidate values, not a
promise of simultaneously readable panes at 24 columns.

## Runtime

`ReadOnlyCollector` receives typed reason, generation, epoch and a cooperative
deadline/cancel control, without a writer/terminal/command callback. This is an
audited capability contract, **not** Rust process/permission isolation. Product
adapters must not capture writer handles, and must honor the bounded admission
contract. The runtime does not implement actual Codex/Agy commands.

`RefreshController` retains one worker/JoinHandle, one active request and at most
one pending request. Timer polling cannot invalidate active completion or replace
an explicit pending Manual; Manual supersedes older generations. Product calls
`advance_epoch` after its own successful transaction; older epochs are rejected.
Snapshots privately expose immutable values, source/scope/sample/freshness and
content revision. Same facts/provenance/epoch retain revision, while updated
sampling metadata does not imply real currentness. `changed` is content change;
the UI must also render actual Loading/Error/state transitions when appropriate.
Known empty differs from not-yet-loaded. Last accepted values survive errors and
Loading carries stale/error status. TTL never upgrades Unknown provenance.

`shutdown(budget)` signals cancellation, stops new requests and joins actual
completion. Timeout reports `ShutdownTimedOut` and retains ownership; caller must
keep/retry the controller until exit. Drop performs a last-resort join and can
block for a collector violating admission. No drop/detach is a cancellation
proof, and no global arbitrary-thread termination is promised.
`OwnedChildLifecycle` is a product-owned terminate/reap protocol. Actual command,
process group/job ownership, stdout limits and stage budgets belong to the
product. A test-only child is launched from the test executable, output-confirmed,
terminated and reaped. No real Provider or Hook is run.

## Documents, themes and terminal

`UiDocument` holds stable facts, verdict, supplied severity, source/scope/time,
freshness and safe next steps. `PermittedText::from_product_redaction` is an
explicit product trust boundary: remove private payloads, commands and secrets
before constructing it. TUC cannot infer whether an arbitrary string is secret.
Both Smart/Raw borrow all the same facts and revision; Raw adds only these
permitted fields. Unknown/FAIL/BLOCKED/STATIC_PASS remain distinct and accessible.
F2 (or user remap) changes only `ViewState`, never collection or persisted state.
When projecting `Snapshot<UiDocument>`, the product binds presentation revision
to the accepted snapshot revision, as demonstrated in the slow-frame fixture.

Neutral semantic/component/surface tokens support none/16/256/truecolor;
`NO_COLOR` takes precedence over Auto/Always/Never. Text status, focus border,
selection marker and row position work without color. Shared message catalog
supports en-US/zh-CN and plural-safe row counts; product labels are supplied data.
No CSIS styles or Plot dependencies are introduced.

`TerminalGuard` owns one terminal session, optionally mouse capture and bracketed
paste. It records attempted operations before entry, restores in reverse on
partial failure and unwind, reports explicit finish errors and retries remaining
cleanup in Drop. The caller must own the terminal; nested foreign raw/alternate
sessions are not admitted. Synthetic ops tests do not qualify a physical terminal.
