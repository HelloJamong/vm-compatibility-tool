# TODO / Work Plan — post `beta-v26.04.01.0003`

## Requirements Summary
- Re-define the Windows registry key list for VBS/core-isolation disabling using the legacy WPF implementation as the source baseline.
- Improve the Tauri/Svelte GUI so it matches or exceeds the readability/usability of the legacy WPF UI.
- Keep the implementation small, reviewable, and verification-driven.

## Evidence Snapshot
- Legacy full VBS/core-isolation disable list: `MainWindow.xaml.cs:3199-3315`
- Legacy detailed registry inspection list: `MainWindow.xaml.cs:4960-5075`
- Current Rust disable list: `src-tauri/src/commands/disable.rs:7-26`
- Current Tauri GUI panels: `src/App.svelte:360-560`
- Legacy WPF layout baseline: `MainWindow.xaml:1-208`

## Gap Summary
1. Current Rust registry lists are materially smaller than the WPF baseline.
2. Current selective execution is category-based, not registry-manifest-based.
3. Current Tauri UI is functional, but still behind WPF in layout density, hierarchy clarity, and operator guidance.

## Acceptance Criteria
- A canonical registry manifest exists for VBS/core-isolation disable work, with each key classified as keep/remove/conditional and rationale documented.
- Rust disable logic uses the canonical manifest instead of ad-hoc hard-coded subsets.
- Virtualization inspection and disable execution share the same registry vocabulary where possible.
- GUI provides stronger visual hierarchy, clearer summaries/actions, and improved readability versus the current `App.svelte`.
- Manual Windows QA checklist exists for registry changes + GUI flows.

## TODO List

### Epic 1 — Registry manifest redefinition (highest priority)
- [ ] Extract the full legacy registry inventory from WPF VBS/core-isolation code.
- [ ] Classify each registry item into:
  - keep in Rust
  - drop as runtime/status-only noise
  - conditional/Windows-version-specific
- [ ] Explicitly review legacy-only keys currently missing in Rust, including:
  - `RequireMicrosoftSignedBootChain`
  - `WasEnabledBy`
  - `SystemGuard`
  - `SecureBiometrics`
  - `LsaCfgFlagsDefault`
  - policy keys like `RequirePlatformSecurityFeatures`
  - `HVCIMATRequired`
  - `ConfigureSystemGuardLaunch`
- [ ] Decide whether `KeyGuard\Status` keys remain intentionally excluded (likely runtime/status keys).
- [ ] Move registry definitions into a canonical shared model/module instead of embedding raw arrays only in `disable.rs`.
- [ ] Align virtualization inspection labels/details with the same manifest so detection and action refer to the same source of truth.
- [ ] Add unit-testable coverage for manifest shape / key classification where feasible.
- [ ] Produce a Windows manual verification matrix for changed registry writes.

### Epic 2 — GUI readability and WPF-parity/improvement
- [ ] Define UI goals from WPF baseline:
  - fast scanability
  - clear primary actions
  - stable table readability
  - explicit risk/warning affordances
- [ ] Refactor `App.svelte` into smaller presentational sections/components before major restyling.
- [ ] Improve layout hierarchy:
  - stronger page titles/subtitles
  - card/section framing
  - consistent action bar placement
  - clearer status badges
- [ ] Improve virtualization results UX:
  - summary cards for “조치 필요 / 정상 / 확인 불가”
  - filters or grouped sections
  - clearer recommendation emphasis
- [ ] Improve disable panel UX:
  - preflight checklist
  - explicit “selected actions” summary
  - better progress/log panel typography
  - clearer success/failure/end-state separation
- [ ] Improve system info panel UX:
  - category grouping that stays readable on long tables
  - quick summary at top (OS/CPU/RAM/가상화)
- [ ] Replace bare `window.confirm` reboot flow with in-app modal/panel UX.
- [ ] Add empty/loading/error states with consistent styling.
- [ ] Validate that the final UX is at least as clear as WPF on 1100x750 and minimum 900x600.

### Epic 3 — Verification and release readiness
- [ ] Create manual QA checklist for Windows admin-run scenarios.
- [ ] Verify selective disable behavior after registry-manifest changes.
- [ ] Verify CSV export, event log readability, reboot request, and no-op disable cases.
- [ ] Update `README.md` / `.claude/CLAUDE.md` after implementation so docs match the product.

## Implementation Steps
1. Audit legacy WPF registry inventory and produce a canonical manifest proposal.
2. Update Rust models/services/commands so registry inspection + disable logic use the new manifest.
3. Add/adjust tests and a Windows manual verification checklist.
4. Split and redesign the Svelte UI with WPF parity as the floor and clearer operator UX as the target.
5. Run verification and then update docs/changelog for the next beta.

## Risks / Mitigations
- Risk: blindly restoring legacy keys may reintroduce ineffective runtime/status writes.
  - Mitigation: classify every legacy key before implementation; keep rationale.
- Risk: GUI cleanup in a single-file `App.svelte` becomes hard to review.
  - Mitigation: do a structure split first, then styling/UX pass.
- Risk: Linux container cannot fully verify Windows-specific behavior.
  - Mitigation: define explicit Windows QA steps and keep platform-dependent verification honest.

## Verification Plan
- `npm run check`
- Rust/unit checks where host environment permits
- Windows manual verification:
  - virtualization scan
  - selective disable
  - full disable
  - reboot request
  - CSV export
  - event log rendering
