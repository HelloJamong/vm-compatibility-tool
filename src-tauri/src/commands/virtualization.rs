/// 가상화 설정 점검 커맨드
///
/// Registry + 프로세스(dism, bcdedit) + WMI 기반 점검
use crate::models::virtualization::{
    DisableGroup, VirtualizationItem, VirtualizationKind, VirtualizationSource,
};
use crate::services::{
    process_service,
    registry_manifest::{self, RegistryAction, RegistryManifestEntry},
    registry_service::windows as reg,
};

enum FeatureState {
    Enabled,
    Disabled,
    Unknown(String),
}

struct RegistryRead {
    value: Option<u32>,
    error: Option<String>,
    detail: String,
}

fn feature_state(result: &process_service::ProcessResult) -> FeatureState {
    if !result.success {
        let message = if !result.stderr.trim().is_empty() {
            result.stderr.trim()
        } else if !result.stdout.trim().is_empty() {
            result.stdout.trim()
        } else {
            "DISM 실행 실패"
        };
        return FeatureState::Unknown(message.to_string());
    }

    match process_service::parse_dism_feature_state(&result.stdout).as_deref() {
        Some(state) if state.eq_ignore_ascii_case("Enabled") => FeatureState::Enabled,
        Some(state) if state.eq_ignore_ascii_case("Disabled") => FeatureState::Disabled,
        Some(state) => FeatureState::Unknown(format!("알 수 없는 DISM 상태: {state}")),
        None => FeatureState::Unknown("DISM 상태 출력을 해석하지 못했습니다".to_string()),
    }
}

#[tauri::command]
pub async fn get_virtualization_status() -> Result<Vec<VirtualizationItem>, String> {
    tokio::task::spawn_blocking(collect_virtualization_status)
        .await
        .map_err(|e| format!("작업 실행 오류: {e}"))?
        .map_err(|e| e.to_string())
}

fn collect_virtualization_status() -> anyhow::Result<Vec<VirtualizationItem>> {
    let mut items = Vec::new();

    check_hardware_virtualization(&mut items);
    check_hyperv_status(&mut items);
    check_wsl_status(&mut items);
    check_hypervisor_launch(&mut items);
    check_vsm_launch(&mut items);
    check_registry_manifest_status(&mut items);
    check_windows_hello_status(&mut items);
    check_organization_control(&mut items);

    Ok(items)
}

// ── 하드웨어 가상화 (WMI Win32_Processor) ─────────────────────────────────

#[cfg(windows)]
fn check_hardware_virtualization(items: &mut Vec<VirtualizationItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::poc_get_cpu_info() {
        Ok(processors) => {
            if let Some(cpu) = processors.into_iter().next() {
                let enabled = cpu.virtualization_firmware_enabled;
                items.push(
                    VirtualizationItem::new(
                        "하드웨어 가상화 (VT-x/AMD-V)",
                        if enabled {
                            "지원됨 (활성)"
                        } else {
                            "비활성화됨"
                        },
                        &format!("CPU: {}", cpu.name),
                        if !enabled {
                            "BIOS/UEFI에서 가상화 옵션을 활성화하세요"
                        } else {
                            ""
                        },
                    )
                    .with_source(VirtualizationSource::Wmi),
                );
            }
        }
        Err(error) => {
            items.push(
                VirtualizationItem::new(
                    "하드웨어 가상화 (VT-x/AMD-V)",
                    "확인 불가",
                    &error.to_string(),
                    "",
                )
                .with_source(VirtualizationSource::Wmi)
                .with_unknown(true),
            );
        }
    }
}

#[cfg(not(windows))]
fn check_hardware_virtualization(items: &mut Vec<VirtualizationItem>) {
    items.push(
        VirtualizationItem::new(
            "하드웨어 가상화 (VT-x/AMD-V)",
            "확인 불가",
            "Windows 전용 기능",
            "",
        )
        .with_source(VirtualizationSource::Wmi)
        .with_unknown(true),
    );
}

// ── Hyper-V 상태 (DISM) ────────────────────────────────────────────────────

fn check_hyperv_status(items: &mut Vec<VirtualizationItem>) {
    let features = [
        ("Microsoft-Hyper-V-All", "Hyper-V (전체)"),
        ("Microsoft-Hyper-V-Hypervisor", "Hyper-V 하이퍼바이저"),
    ];

    for (feature, label) in features {
        let result = process_service::get_feature_state(feature);
        let state = feature_state(&result);
        let is_unknown = matches!(&state, FeatureState::Unknown(_));
        let (status, details, rec, action_required) = match state {
            FeatureState::Enabled => (
                "설치됨 (활성)",
                format!("{feature} 가 활성화되어 있습니다"),
                "VM 사용을 위해 비활성화가 필요합니다",
                true,
            ),
            FeatureState::Disabled => (
                "설치됨 (비활성)",
                format!("{feature} 가 비활성화되어 있습니다"),
                "",
                false,
            ),
            FeatureState::Unknown(message) => (
                "확인 불가",
                message,
                "DISM 상태를 확인한 뒤 다시 점검하세요",
                false,
            ),
        };

        items.push(
            VirtualizationItem::new(label, status, &details, rec)
                .with_source(VirtualizationSource::Feature)
                .with_disable_group(DisableGroup::Hyperv, action_required)
                .with_unknown(is_unknown),
        );
    }
}

// ── WSL 상태 (DISM) ────────────────────────────────────────────────────────

fn check_wsl_status(items: &mut Vec<VirtualizationItem>) {
    let features = [
        ("Microsoft-Windows-Subsystem-Linux", "WSL"),
        ("VirtualMachinePlatform", "가상 머신 플랫폼 (WSL2)"),
    ];

    for (feature, label) in features {
        let result = process_service::get_feature_state(feature);
        let state = feature_state(&result);
        let is_unknown = matches!(&state, FeatureState::Unknown(_));
        let (status, details, rec, action_required) = match state {
            FeatureState::Enabled => (
                "설치됨 (활성)",
                format!("{label} 가 활성화되어 있습니다"),
                "VM 성능 향상을 위해 비활성화를 권장합니다",
                true,
            ),
            FeatureState::Disabled => (
                "비활성 또는 미설치",
                format!("{label} 가 비활성화되어 있습니다"),
                "",
                false,
            ),
            FeatureState::Unknown(message) => (
                "확인 불가",
                message,
                "DISM 상태를 확인한 뒤 다시 점검하세요",
                false,
            ),
        };

        items.push(
            VirtualizationItem::new(label, status, &details, rec)
                .with_source(VirtualizationSource::Feature)
                .with_disable_group(DisableGroup::Wsl, action_required)
                .with_unknown(is_unknown),
        );
    }
}

// ── Hypervisor 시작 유형 (bcdedit) ─────────────────────────────────────────

fn check_hypervisor_launch(items: &mut Vec<VirtualizationItem>) {
    let launch_type = process_service::get_hypervisor_launch_type();
    let is_unknown =
        launch_type.eq_ignore_ascii_case("확인 불가") || launch_type.starts_with("오류");
    let is_active = !is_unknown && !launch_type.eq_ignore_ascii_case("off");

    items.push(
        VirtualizationItem::new(
            "Hypervisor 시작 유형",
            &launch_type,
            &format!("bcdedit hypervisorlaunchtype: {launch_type}"),
            if is_active {
                "비활성화를 위해 bcdedit /set hypervisorlaunchtype off 실행 필요"
            } else {
                ""
            },
        )
        .with_source(VirtualizationSource::Bcd)
        .with_disable_group(DisableGroup::Hyperv, is_active)
        .with_unknown(is_unknown),
    );
}

// ── VSM 시작 유형 (bcdedit vsmlaunchtype) ──────────────────────────────────

fn check_vsm_launch(items: &mut Vec<VirtualizationItem>) {
    let vsm_type = process_service::get_vsm_launch_type();
    let is_unknown = vsm_type.eq_ignore_ascii_case("확인 불가") || vsm_type.starts_with("오류");
    let is_active = !is_unknown && !matches!(vsm_type.to_lowercase().as_str(), "off" | "미설정");

    items.push(
        VirtualizationItem::new(
            "VSM 시작 유형 (vsmlaunchtype)",
            &vsm_type,
            &format!("bcdedit vsmlaunchtype: {vsm_type}"),
            if is_active {
                "비활성화를 위해 bcdedit /set vsmlaunchtype off 실행 필요"
            } else {
                ""
            },
        )
        .with_source(VirtualizationSource::Bcd)
        .with_disable_group(DisableGroup::Hyperv, is_active)
        .with_unknown(is_unknown),
    );
}

// ── 레지스트리 기반 VBS / 코어 격리 상태 ───────────────────────────────────

fn check_registry_manifest_status(items: &mut Vec<VirtualizationItem>) {
    for entry in registry_manifest::inspect_entries() {
        let values = registry_manifest::resolve_entry_paths(entry)
            .into_iter()
            .map(
                |resolved| match reg::get_dword_result(&resolved.path, resolved.value_name) {
                    Ok(Some(value)) => RegistryRead {
                        value: Some(value),
                        error: None,
                        detail: format!(r"{}\{} = {}", resolved.path, resolved.value_name, value),
                    },
                    Ok(None) => RegistryRead {
                        value: None,
                        error: None,
                        detail: format!(r"{}\{} = <미설정>", resolved.path, resolved.value_name),
                    },
                    Err(error) => RegistryRead {
                        value: None,
                        error: Some(error.to_string()),
                        detail: format!(
                            r"{}\{} = <확인 불가: {}>",
                            resolved.path, resolved.value_name, error
                        ),
                    },
                },
            )
            .collect::<Vec<_>>();

        items.push(build_registry_item(entry, &values));
    }
}

fn build_registry_item(
    entry: &RegistryManifestEntry,
    values: &[RegistryRead],
) -> VirtualizationItem {
    let details = values
        .iter()
        .map(|read| read.detail.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    match entry.action {
        RegistryAction::DisableWrite => build_disable_write_registry_item(entry, values, &details),
        RegistryAction::InspectOnly => {
            VirtualizationItem::new(entry.label, &registry_inspect_status(values), &details, "")
                .with_source(VirtualizationSource::Registry)
                .with_disable_group(entry.disable_group, false)
                .with_unknown(has_registry_error(values))
                .with_manifest_id(entry.id)
        }
        RegistryAction::ExcludedLegacy => {
            build_excluded_legacy_registry_item(entry, values, &details)
        }
    }
}

fn build_disable_write_registry_item(
    entry: &RegistryManifestEntry,
    values: &[RegistryRead],
    details: &str,
) -> VirtualizationItem {
    let target_value = entry.target_value.unwrap_or(0);
    let action_required = values
        .iter()
        .filter_map(|read| read.value)
        .any(|value| value != target_value);
    let has_any_value = values.iter().any(|read| read.value.is_some());
    let has_error = has_registry_error(values);

    let status = if has_error {
        "확인 불가"
    } else if action_required {
        "활성화됨"
    } else if has_any_value {
        "비활성화됨"
    } else {
        "미설정"
    };

    let recommendation = if action_required {
        "비활성화가 필요합니다"
    } else {
        ""
    };

    VirtualizationItem::new(entry.label, status, details, recommendation)
        .with_source(VirtualizationSource::Registry)
        .with_disable_group(entry.disable_group, action_required)
        .with_unknown(has_error)
        .with_manifest_id(entry.id)
}

fn has_registry_error(values: &[RegistryRead]) -> bool {
    values.iter().any(|read| read.error.is_some())
}

fn registry_inspect_status(values: &[RegistryRead]) -> String {
    if has_registry_error(values) {
        return "확인 불가".to_string();
    }

    values
        .iter()
        .filter_map(|read| read.value)
        .next()
        .map(|value| format!("값: {value}"))
        .unwrap_or_else(|| "미설정".to_string())
}

fn build_excluded_legacy_registry_item(
    entry: &RegistryManifestEntry,
    values: &[RegistryRead],
    details: &str,
) -> VirtualizationItem {
    let target_value = entry.target_value.unwrap_or(0);
    let has_any_value = values.iter().any(|read| read.value.is_some());
    let differs_from_target = values
        .iter()
        .filter_map(|read| read.value)
        .any(|value| value != target_value);
    let has_error = has_registry_error(values);

    let status = if has_error {
        "확인 불가"
    } else if differs_from_target {
        "활성화됨 (참고)"
    } else if has_any_value {
        "비활성화됨 (참고)"
    } else {
        "미설정 (참고)"
    };

    let recommendation = if differs_from_target {
        "자동 조치 제외 항목 — 필요 시 수동 검토"
    } else {
        "참고용 항목"
    };

    VirtualizationItem::new(entry.label, status, details, recommendation)
        .with_source(VirtualizationSource::Registry)
        .with_disable_group(entry.disable_group, false)
        .with_optional_action_available(differs_from_target && !has_error)
        .with_unknown(has_error)
        .with_manifest_id(entry.id)
}

// ── Windows Hello / WHfB 상태 ─────────────────────────────────────────────

fn check_windows_hello_status(items: &mut Vec<VirtualizationItem>) {
    if !is_windows_hello_active() {
        return;
    }

    let Some(whfb_type) = detect_whfb_type() else {
        return; // 기본 Windows Hello — VBS 무관
    };

    let (can_disable, disable_reason) = check_whfb_disableable();

    let status = if can_disable {
        "WHfB 활성 — 해제 가능"
    } else {
        "WHfB 활성 — 해제 불가"
    };

    let recommendation = if can_disable {
        "VBS 비활성화 전 해제 권장: 설정 → 계정 → 회사 또는 학교 액세스 → 연결 끊기".to_string()
    } else {
        format!(
            "VBS 설정이 재부팅 후 복구될 수 있습니다 — {}",
            disable_reason
        )
    };

    items.push(
        VirtualizationItem::new(
            "Windows Hello",
            status,
            &format!("감지 유형: {}", whfb_type),
            &recommendation,
        )
        .with_source(VirtualizationSource::Registry)
        .with_kind(VirtualizationKind::WhfbWarning),
    );
}

#[cfg(windows)]
fn is_windows_hello_active() -> bool {
    use std::path::Path;
    let ngc = Path::new(r"C:\Windows\ServiceProfiles\LocalService\AppData\Local\Microsoft\Ngc");
    ngc.exists()
        && ngc
            .read_dir()
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_windows_hello_active() -> bool {
    false
}

#[cfg(windows)]
fn detect_whfb_type() -> Option<String> {
    let aad_joined =
        reg::key_has_subkeys(r"SYSTEM\CurrentControlSet\Control\CloudDomainJoin\JoinInfo");
    let policy_enabled = reg::get_dword(r"SOFTWARE\Policies\Microsoft\PassportForWork", "Enabled")
        .map(|v| v == 1)
        .unwrap_or(false);
    let mdm_enrolled = has_mdm_corporate_enrollment();

    if policy_enabled {
        Some("GPO/MDM 정책 적용".to_string())
    } else if aad_joined {
        Some("Azure AD 조인".to_string())
    } else if mdm_enrolled {
        Some("MDM 등록".to_string())
    } else {
        None
    }
}

#[cfg(not(windows))]
fn detect_whfb_type() -> Option<String> {
    None
}

#[cfg(windows)]
fn check_whfb_disableable() -> (bool, String) {
    let policy_enabled = reg::get_dword(r"SOFTWARE\Policies\Microsoft\PassportForWork", "Enabled")
        .map(|v| v == 1)
        .unwrap_or(false);

    if policy_enabled {
        return (
            false,
            "GPO/MDM 정책으로 강제 적용 중 — IT 관리자 확인 필요".to_string(),
        );
    }

    if has_mdm_corporate_enrollment() {
        return (
            false,
            "기업 MDM 관리 기기 — IT 관리자 확인 필요".to_string(),
        );
    }

    (true, String::new())
}

#[cfg(not(windows))]
fn check_whfb_disableable() -> (bool, String) {
    (false, String::new())
}

#[cfg(windows)]
fn has_mdm_corporate_enrollment() -> bool {
    reg::list_subkeys(r"SOFTWARE\Microsoft\Enrollments")
        .into_iter()
        .any(|subkey| {
            let path = format!(r"SOFTWARE\Microsoft\Enrollments\{}", subkey);
            // EnrollmentType 6 = MDM, 13 = AAD MDM
            matches!(reg::get_dword(&path, "EnrollmentType"), Some(6) | Some(13))
        })
}

#[cfg(not(windows))]
fn has_mdm_corporate_enrollment() -> bool {
    false
}

// ── 조직 관리 장치 감지 (AAD / MDM) ──────────────────────────────────────

fn check_organization_control(items: &mut Vec<VirtualizationItem>) {
    let (is_org_managed, org_type) = detect_organization_control();
    if !is_org_managed {
        return;
    }

    items.push(
        VirtualizationItem::new(
            "조직 관리 장치",
            &format!("감지됨 — {}", org_type),
            &format!("조직 연결 유형: {}", org_type),
            "비활성화 후 재부팅 시 VBS 설정이 정책으로 재적용될 수 있습니다 — IT 관리자 확인 권장",
        )
        .with_source(VirtualizationSource::Registry)
        .with_kind(VirtualizationKind::OrganizationWarning),
    );
}

#[cfg(windows)]
fn detect_organization_control() -> (bool, String) {
    let aad_joined =
        reg::key_has_subkeys(r"SYSTEM\CurrentControlSet\Control\CloudDomainJoin\JoinInfo");
    let mdm_enrolled = has_mdm_corporate_enrollment();

    if aad_joined && mdm_enrolled {
        (true, "Azure AD 조인 + MDM 등록".to_string())
    } else if aad_joined {
        (true, "Azure AD 조인".to_string())
    } else if mdm_enrolled {
        (true, "MDM 등록".to_string())
    } else {
        (false, String::new())
    }
}

#[cfg(not(windows))]
fn detect_organization_control() -> (bool, String) {
    (false, String::new())
}

#[cfg(test)]
mod tests {
    use super::{feature_state, registry_inspect_status, FeatureState, RegistryRead};
    use crate::services::process_service::ProcessResult;

    fn process_result(success: bool, stdout: &str, stderr: &str) -> ProcessResult {
        ProcessResult {
            success,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code: if success { 0 } else { 1 },
        }
    }

    #[test]
    fn feature_state_detects_enabled_dism_output() {
        let result = process_result(true, "Feature Name : Example\r\nState : Enabled\r\n", "");
        assert!(matches!(feature_state(&result), FeatureState::Enabled));
    }

    #[test]
    fn feature_state_preserves_dism_failure_as_unknown() {
        let result = process_result(false, "", "Access denied");
        match feature_state(&result) {
            FeatureState::Unknown(message) => assert_eq!(message, "Access denied"),
            _ => panic!("DISM failure must not be reported as disabled"),
        }
    }

    #[test]
    fn feature_state_rejects_unexpected_success_output() {
        let result = process_result(true, "The operation completed successfully.", "");
        assert!(matches!(feature_state(&result), FeatureState::Unknown(_)));
    }

    #[test]
    fn registry_read_errors_are_not_reported_as_missing() {
        let values = vec![RegistryRead {
            value: None,
            error: Some("Access denied".to_string()),
            detail: "Example = <확인 불가>".to_string(),
        }];

        assert_eq!(registry_inspect_status(&values), "확인 불가");
    }
}
