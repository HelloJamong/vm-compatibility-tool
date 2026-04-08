# Design Plan — Registry Manifest Refactor + GUI Overhaul

## Scope
- Rebuild the VBS / core-isolation registry handling around a single canonical manifest.
- Use that manifest as the shared source of truth for both inspection and disable execution.
- Redesign the Tauri/Svelte UI so it reaches or exceeds the clarity of the legacy WPF UI.

## Evidence
- Current disable registry arrays: `src-tauri/src/commands/disable.rs:7-26`
- Current disable flow/event model: `src-tauri/src/commands/disable.rs:47-118`
- Current virtualization data model: `src-tauri/src/models/virtualization.rs:1-58`
- Current frontend action derivation: `src/App.svelte:193-217`
- Current frontend screens: `src/App.svelte:360-560`
- Legacy WPF UI baseline: `MainWindow.xaml:1-208`
- Legacy VBS/core-isolation write set: `MainWindow.xaml.cs:3199-3315`
- Legacy detailed registry inspection list: `MainWindow.xaml.cs:4960-5075`

## Problem Statement
1. Registry keys are hard-coded in `disable.rs` and currently cover only a subset of the legacy behavior.
2. Inspection and disable execution do not share the same registry vocabulary/source of truth.
3. The frontend infers actions by matching Korean label strings, which is brittle.
4. The current single-file `App.svelte` UI is functional but not yet at WPF-grade readability.

---

## Canonical Registry Design

### 1. Registry entry model
Create one canonical manifest module for all registry-backed VM compatibility rules.

### Proposed file
- `src-tauri/src/services/registry_manifest.rs`

### Proposed types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisableGroup {
    Vbs,
    CoreIsolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryAction {
    DisableWrite,
    InspectOnly,
    ExcludedLegacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryHiveSet {
    CurrentOnly,
    CurrentAndControlSet001,
    PoliciesOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct RegistryManifestEntry {
    pub id: &'static str,
    pub path: &'static str,
    pub value_name: &'static str,
    pub target_value: Option<u32>,
    pub disable_group: DisableGroup,
    pub action: RegistryAction,
    pub hive_set: RegistryHiveSet,
    pub label: &'static str,
    pub rationale: &'static str,
}
```

### Why this shape
- `action` separates real write targets from inspect-only values like `WasEnabledBy`.
- `disable_group` allows selective disable to stop depending on UI label matching.
- `hive_set` makes `ControlSet001` mirroring explicit instead of manually duplicated everywhere.
- `rationale` preserves why a key is included/excluded.

---

## Manifest classification

### A. `DisableWrite` set (default 0 write targets)
These should be the main active write targets.

#### VBS / DeviceGuard
- `EnableVirtualizationBasedSecurity`
- `RequirePlatformSecurityFeatures`
- `Locked`
- `Mandatory`

#### HVCI
- `HypervisorEnforcedCodeIntegrity\\Enabled`
- `HypervisorEnforcedCodeIntegrity\\Locked`

#### Credential Guard / LSA
- `CredentialGuard\\Enabled`
- `LsaCfgFlags`

#### Policy
- `EnableVirtualizationBasedSecurity`
- `RequirePlatformSecurityFeatures`
- `LsaCfgFlags`
- `HypervisorEnforcedCodeIntegrity`
- `HVCIEnabled`
- `HVCIMATRequired`

#### Core isolation support
- `CI\\Config\\VulnerableDriverBlocklistEnable`

### B. `InspectOnly` set
Keep visible in scans, but do not write during disable.
- `HypervisorEnforcedCodeIntegrity\\WasEnabledBy`

### C. `ExcludedLegacy` set
Document them in the manifest but exclude from the default write path.
- `RequireMicrosoftSignedBootChain`
- `SystemGuard\\Enabled`
- `SecureBiometrics\\Enabled`
- `LsaCfgFlagsDefault`
- `ConfigureSystemGuardLaunch`
- `KeyGuard\\Status\\*`

### D. ControlSet mirroring rule
Apply `CurrentControlSet + ControlSet001` mirroring only for:
- `SYSTEM\\...\\DeviceGuard`
- `SYSTEM\\...\\Lsa`
- scenario subkeys under `SYSTEM`

Do **not** mirror policy keys under `SOFTWARE\\Policies\\...`.

---

## Backend Refactor Design

### 2. Registry write helpers
### Existing files
- `src-tauri/src/services/registry_service.rs`
- `src-tauri/src/commands/disable.rs`
- `src-tauri/src/commands/virtualization.rs`

### Planned changes
1. Add manifest expansion helpers:
   - `all_manifest_entries()`
   - `disable_write_entries(group: DisableGroup)`
   - `inspect_entries()`
   - `expanded_entries()` for `ControlSet001` mirroring
2. Move raw arrays out of `disable.rs`.
3. Replace `disable_vbs()` / `disable_core_isolation()` loops with manifest-driven iteration.
4. Reuse the same manifest for virtualization registry inspection.

### Result
- One place to update registry behavior.
- Inspection and disable remain consistent.
- Easier Windows-version maintenance.

---

## Data Model Design

### 3. `VirtualizationItem` enrichment
### Existing file
- `src-tauri/src/models/virtualization.rs`

### Proposed extension
```rust
pub struct VirtualizationItem {
    pub category: String,
    pub status: String,
    pub details: String,
    pub recommendation: String,
    pub disable_group: Option<String>,
    pub source_type: String,      // feature | registry | bcd | wmi
    pub action_required: bool,
    pub manifest_id: Option<String>,
}
```

### Why
- Frontend should use structured flags, not string heuristics.
- `manifest_id` links inspection rows back to registry entries when needed.
- `action_required` makes summary cards and selective preview trivial.

### Compatibility strategy
- Keep existing four display fields.
- Add new fields as non-breaking enrichments.

---

## Selective Disable Design

### 4. Source of truth for action preview
### Current problem
`src/App.svelte:193-217` derives actions by matching visible text labels.

### New design
- Backend sets `disable_group` and `action_required` for each inspection result.
- Frontend computes preview by grouping `virtItems` on `disable_group`.
- Optional later step: backend can expose a dedicated `compute_disable_plan` command, but this is not required for the first pass.

### Outcome
- No dependency on Korean label wording.
- Registry additions/removals do not force fragile UI string updates.

---

## GUI Overhaul Design

### 5. Component split
### Current file
- `src/App.svelte`

### Proposed split
- `src/components/layout/AppHeader.svelte`
- `src/components/layout/StatusBar.svelte`
- `src/components/menu/MenuPanel.svelte`
- `src/components/system/SystemInfoPanel.svelte`
- `src/components/virtualization/VirtualizationPanel.svelte`
- `src/components/disable/DisablePanel.svelte`
- `src/components/common/SummaryCard.svelte`
- `src/components/common/StatusBadge.svelte`
- `src/components/common/EmptyState.svelte`
- `src/components/common/ConfirmDialog.svelte`

### Why split first
- Styling and UX changes become reviewable.
- Each panel can evolve without making `App.svelte` unmanageable.

### 6. UX targets by panel
#### Menu
- Add short descriptions under each primary action.
- Show last scan state / action-needed badge if available.

#### System info
- Add top summary strip: OS / CPU / RAM / virtualization-ready.
- Keep grouped rows, but improve category visual anchors.

#### Virtualization
- Add summary cards:
  - 조치 필요
  - 정상
  - 확인 불가
- Keep the table, but improve row emphasis for risky items.
- Make “비활성화 실행” CTA more visible and context-driven.

#### Disable
- Replace plain warning block with:
  - preflight warning card
  - selected-actions summary card
  - execution log card
- Replace `window.confirm` with in-app confirmation dialog.
- Improve progress states: queued / running / partial-failure / complete.

#### Footer/status
- Keep simple WPF-like status bar behavior, but add clearer transient state messaging.

---

## Phased Execution Plan

### Phase 1 — Manifest specification
- Build the canonical registry manifest.
- Mark every legacy key as write / inspect / excluded.
- Document rationale inline.

### Phase 2 — Backend refactor
- Add `registry_manifest.rs`.
- Refactor disable execution to use manifest-driven iteration.
- Refactor virtualization registry inspection to use manifest-driven reads where applicable.
- Extend `VirtualizationItem` with structured fields.

### Phase 3 — Frontend behavior cleanup
- Remove string-based selective action inference.
- Use `disable_group` + `action_required` from backend.
- Keep the current UI layout temporarily while behavior is stabilized.

### Phase 4 — UI restructure + redesign
- Split `App.svelte` into components.
- Add summary cards, action bars, dialog, improved table/readability patterns.
- Match WPF parity first, then improve beyond it.

### Phase 5 — Verification + docs
- Run `npm run check`.
- Run Rust checks where host permits.
- Execute Windows manual QA for scan/disable/export/reboot paths.
- Update `README.md`, `.claude/CLAUDE.md`, and `CHANGELOG.md` for the next beta.

---

## Acceptance Criteria
- Registry writes are driven by one canonical manifest.
- Current Rust registry behavior is no longer a partial subset without rationale.
- Inspect-only values like `WasEnabledBy` are visible but not written.
- Frontend preview logic does not depend on hard-coded display strings.
- UI is componentized and more readable than the current single-file implementation.
- Windows QA checklist exists and is executed before the next beta cut.

## Risks
- Over-restoring legacy keys may reintroduce ineffective runtime writes.
- UI refactor may create avoidable churn if done before backend structure stabilizes.
- Linux environment cannot prove Windows-specific behavior.

## Mitigations
- Keep an explicit `ExcludedLegacy` list in code comments/docs.
- Finish backend manifest refactor before visual redesign.
- Use a manual Windows verification matrix as a required release gate.

## Verification Plan
- `npm run check`
- Rust unit/compile checks as available on host
- Windows manual verification:
  - virtualization scan
  - selective disable
  - full disable
  - reboot confirmation
  - CSV export
  - event log readability
