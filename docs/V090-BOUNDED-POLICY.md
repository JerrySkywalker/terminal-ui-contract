# v0.9.0 consumer policy and candidate API boundaries

Decision TUI-V090-BOUNDED-SUPPORT-001 is Owner-approved product policy;
OWNER_APPROVED_V090_BOUNDED_POLICY != OWNER_FROZEN_TUC_PUBLIC_API.
Core 0.2.0, Runtime 0.1.0 and Ratatui 0.1.0 remain unreleased API candidates.
No signature, version, permission default or worker/Drop model changes follow.

| Area | Candidate boundary |
| --- | --- |
| Core / Adapter | Core remains backend/domain neutral; Adapter alone exposes Ratatui/Crossterm types. Exact public signature freeze is PENDING_OWNER. |
| Standard / Vim / Custom | Standard and Vim presets, product-validated Custom overlay; context owns sequences/counts. Captured modal input does not fall through. Explicit registered product context opt-in yields an intent, never execution or authority. |
| Focus / HitMap / Viewport / CJK | Focus-only transitions, committed geometry and ancestor clips, hover-wheel routing, stale-generation refusal, display-cell/grapheme reflow and bounded scrolling. Physical delivery remains a separate gate. |
| PermittedText | Product redacts secrets, commands, payloads and sensitive paths before construction. The constructor is a trust boundary, not a sanitizer. Smart/Raw use the same approved facts. |
| Runtime | Admitted collectors cooperate with cancellation/deadlines. shutdown bounds caller wait and retains the worker on timeout. Drop performs an unbounded Join; native OS/I/O stalls and noncooperative workers have no strict exit guarantee. |
| TerminalGuard | Attempts entry, reverses cleanup and reports finish errors. Output success is not private-mode state readback; DEC1049/mouse/paste remain UNPROVEN without trusted DECRQM. |
| Timing / human | Software frame submission is not optical input-to-screen. Physical keyboard/mouse/visual and latency/SLA remain independently unqualified. |
| TabBeacon | Real Draft #123 consumes all three exact versions from one exact Git revision. Final manifest/lock/source/CI bindings are in its PR and coordination receipt. No public-registry consumer PASS. |
| HookStat | Reference and labelled synthetic fixture only. No current real-product migration or compatibility claim; no mutation. |

Core -> Runtime -> Ratatui is the future publication order, followed by an exact
public-registry consumer manifest/lock/package qualification. Local-registry
archives are preparation only. Owner API approval, public package resolution,
consumer validation, merge/publication/install/release remain separate gates.
No gate is relaxed by a product-level bounded support decision.

产品有界政策不冻结 TUC 公共 API。Runtime 仅合作取消，Drop Join 可能无界；
TerminalGuard 清理命令成功不证明 DEC 状态恢复。PermittedText 必须由产品先
脱敏。软件帧不是光学延迟，synthetic HookStat 不是实际迁移。
