# Windows Manual QA Checklist

## Scope
Manual verification checklist for the Tauri-based VM Compatibility Tool after the registry-manifest refactor and GUI restructuring.

## Preconditions
- Test on Windows 10/11 with **administrator rights**.
- Prefer a VM or snapshot-capable environment.
- Keep one baseline machine/profile where:
  - Hyper-V / WSL / VBS / HVCI are enabled in some combination
  - some registry values exist and are active
  - some registry values are intentionally missing

## Build / Launch
- [ ] Launch the app as administrator.
- [ ] Confirm the app opens without a console window in release build.
- [ ] Confirm the app title and icon appear correctly.
- [ ] Confirm launching without admin shows the required warning and exits.

## Menu / Navigation
- [ ] Main menu renders all three primary actions.
- [ ] Menu badges render correctly after a virtualization scan.
- [ ] Back navigation returns to the main menu from each panel.

## System Info
- [ ] "시스템 사양 체크" loads successfully.
- [ ] OS / CPU / memory / disk / boot / motherboard / GPU / power sections render.
- [ ] Event log summary renders without mojibake.
- [ ] CSV export for system info succeeds and opens correctly in Excel/Notepad.

## Virtualization Scan
- [ ] "가상화 설정 점검" completes successfully.
- [ ] Summary cards show counts for:
  - 조치 필요
  - 정상/비활성
  - 확인 불가
- [ ] Hyper-V / WSL / hypervisorlaunchtype rows match actual machine state.
- [ ] VBS/HVCI/CredentialGuard/LSA-related registry rows appear as expected.
- [ ] Excluded legacy registry rows appear as **reference-only** rows.
- [ ] Reference-only rows do **not** imply automatic remediation.

## Missing-key Policy
For one or more manifest-backed registry values, remove the value entirely before running the tool.

- [ ] Scan shows the missing value as `미설정` / equivalent missing-state detail.
- [ ] That missing value does **not** force the corresponding action group by itself.
- [ ] Run disable.
- [ ] Confirm the log states the value was missing and **not created**.
- [ ] Re-scan and confirm the value is still missing unless Windows recreated it independently.

## Selective Disable Policy
Prepare a machine where some relevant values exist and are active, and others are missing/inactive.

- [ ] Scan marks only existing active values as action-required.
- [ ] Disable preview selects only the expected groups.
- [ ] Disable execution changes existing active registry values to `0`.
- [ ] Existing inactive values are reported as already inactive.
- [ ] Missing values are skipped and not created.
- [ ] A re-scan reflects the updated inactive state.

## Hyper-V / WSL / BCD
- [ ] Hyper-V features are detected correctly.
- [ ] WSL / VirtualMachinePlatform detection is correct.
- [ ] If hypervisorlaunchtype is active, it is marked action-required.
- [ ] Disable run updates hypervisorlaunchtype to `off` when needed.
- [ ] Re-scan reflects the expected BCD state.

## Disable Panel / Logs
- [ ] Selected action summary matches the scan result.
- [ ] Progress log shows step-by-step execution.
- [ ] Success / warning / skipped / missing-value lines are visually distinguishable.
- [ ] No-op cases show that no disable action was necessary.

## Reboot Flow
- [ ] After a successful disable run, the reboot button is shown.
- [ ] Clicking reboot opens the in-app confirmation dialog.
- [ ] Cancel closes the dialog without side effects.
- [ ] Confirm schedules reboot successfully.

## Regression Checks
- [ ] `npm run check`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`
- [ ] No new obvious layout breakage at 1100x750
- [ ] Minimum supported size 820x560 remains usable

## Notes to Record During QA
- Windows version / build
- Which security features were initially enabled
- Which registry values were present vs missing
- Whether excluded legacy rows were useful/noisy
- Any row labels/details that need wording cleanup
