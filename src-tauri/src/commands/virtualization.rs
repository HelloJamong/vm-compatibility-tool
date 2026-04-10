/// 가상화 설정 점검 커맨드
///
/// Registry + 프로세스(dism, bcdedit) + WMI 기반 점검
use crate::models::virtualization::{DisableGroup, VirtualizationItem, VirtualizationSource};
use crate::services::{
    process_service,
    registry_manifest::{self, RegistryAction, RegistryManifestEntry},
    registry_service::windows as reg,
};

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
    check_registry_manifest_status(&mut items);
    check_windows_hello_status(&mut items);

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
                .with_source(VirtualizationSource::Wmi),
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
        .with_source(VirtualizationSource::Wmi),
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
        let (status, details, rec, action_required) = if !result.success {
            (
                "확인 불가",
                "DISM 실행 실패 — 관리자 권한으로 실행하세요".to_string(),
                "관리자 권한으로 실행하세요",
                false,
            )
        } else if result.stdout.contains("State : Enabled") {
            (
                "설치됨 (활성)",
                format!("{feature} 가 활성화되어 있습니다"),
                "VM 사용을 위해 비활성화가 필요합니다",
                true,
            )
        } else if result.stdout.contains("State : Disabled") {
            (
                "설치됨 (비활성)",
                format!("{feature} 가 비활성화되어 있습니다"),
                "",
                false,
            )
        } else {
            (
                "미설치",
                format!("{feature} 가 설치되어 있지 않습니다"),
                "",
                false,
            )
        };

        items.push(
            VirtualizationItem::new(label, status, &details, rec)
                .with_source(VirtualizationSource::Feature)
                .with_disable_group(DisableGroup::Hyperv, action_required),
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
        let (status, details, rec, action_required) = if result.stdout.contains("State : Enabled") {
            (
                "설치됨 (활성)",
                format!("{label} 가 활성화되어 있습니다"),
                "VM 성능 향상을 위해 비활성화를 권장합니다",
                true,
            )
        } else {
            (
                "비활성 또는 미설치",
                format!("{label} 가 비활성화되어 있습니다"),
                "",
                false,
            )
        };

        items.push(
            VirtualizationItem::new(label, status, &details, rec)
                .with_source(VirtualizationSource::Feature)
                .with_disable_group(DisableGroup::Wsl, action_required),
        );
    }
}

// ── Hypervisor 시작 유형 (bcdedit) ─────────────────────────────────────────

fn check_hypervisor_launch(items: &mut Vec<VirtualizationItem>) {
    let launch_type = process_service::get_hypervisor_launch_type();
    let is_active = !matches!(launch_type.to_lowercase().as_str(), "off" | "확인 불가")
        && !launch_type.starts_with("오류");

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
        .with_disable_group(DisableGroup::Hyperv, is_active),
    );
}

// ── 레지스트리 기반 VBS / 코어 격리 상태 ───────────────────────────────────

fn check_registry_manifest_status(items: &mut Vec<VirtualizationItem>) {
    for entry in registry_manifest::inspect_entries() {
        let values = registry_manifest::resolve_entry_paths(entry)
            .into_iter()
            .map(|resolved| {
                let value = reg::get_dword(&resolved.path, resolved.value_name);
                let detail = match value {
                    Some(current) => {
                        format!(r"{}\{} = {}", resolved.path, resolved.value_name, current)
                    }
                    None => format!(r"{}\{} = <미설정>", resolved.path, resolved.value_name),
                };
                (value, detail)
            })
            .collect::<Vec<_>>();

        items.push(build_registry_item(entry, &values));
    }
}

fn build_registry_item(
    entry: &RegistryManifestEntry,
    values: &[(Option<u32>, String)],
) -> VirtualizationItem {
    let details = values
        .iter()
        .map(|(_, detail)| detail.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    match entry.action {
        RegistryAction::DisableWrite => build_disable_write_registry_item(entry, values, &details),
        RegistryAction::InspectOnly => {
            VirtualizationItem::new(entry.label, &registry_inspect_status(values), &details, "")
                .with_source(VirtualizationSource::Registry)
                .with_disable_group(entry.disable_group, false)
                .with_manifest_id(entry.id)
        }
        RegistryAction::ExcludedLegacy => {
            build_excluded_legacy_registry_item(entry, values, &details)
        }
    }
}

fn build_disable_write_registry_item(
    entry: &RegistryManifestEntry,
    values: &[(Option<u32>, String)],
    details: &str,
) -> VirtualizationItem {
    let target_value = entry.target_value.unwrap_or(0);
    let action_required = values
        .iter()
        .filter_map(|(value, _)| *value)
        .any(|value| value != target_value);
    let has_any_value = values.iter().any(|(value, _)| value.is_some());

    let status = if action_required {
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
        .with_manifest_id(entry.id)
}

fn registry_inspect_status(values: &[(Option<u32>, String)]) -> String {
    values
        .iter()
        .filter_map(|(value, _)| *value)
        .next()
        .map(|value| format!("값: {value}"))
        .unwrap_or_else(|| "미설정".to_string())
}

fn build_excluded_legacy_registry_item(
    entry: &RegistryManifestEntry,
    values: &[(Option<u32>, String)],
    details: &str,
) -> VirtualizationItem {
    let target_value = entry.target_value.unwrap_or(0);
    let has_any_value = values.iter().any(|(value, _)| value.is_some());
    let differs_from_target = values
        .iter()
        .filter_map(|(value, _)| *value)
        .any(|value| value != target_value);

    let status = if differs_from_target {
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
        format!("VBS 설정이 재부팅 후 복구될 수 있습니다 — {}", disable_reason)
    };

    items.push(
        VirtualizationItem::new(
            "Windows Hello",
            status,
            &format!("감지 유형: {}", whfb_type),
            &recommendation,
        )
        .with_source(VirtualizationSource::Registry)
        .with_manifest_id("whfb_check"),
    );
}

#[cfg(windows)]
fn is_windows_hello_active() -> bool {
    use std::path::Path;
    let ngc = Path::new(
        r"C:\Windows\ServiceProfiles\LocalService\AppData\Local\Microsoft\Ngc",
    );
    ngc.exists() && ngc.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
}

#[cfg(not(windows))]
fn is_windows_hello_active() -> bool {
    false
}

#[cfg(windows)]
fn detect_whfb_type() -> Option<String> {
    let aad_joined = reg::key_has_subkeys(
        r"SYSTEM\CurrentControlSet\Control\CloudDomainJoin\JoinInfo",
    );
    let policy_enabled = reg::get_dword(
        r"SOFTWARE\Policies\Microsoft\PassportForWork",
        "Enabled",
    )
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
    let policy_enabled = reg::get_dword(
        r"SOFTWARE\Policies\Microsoft\PassportForWork",
        "Enabled",
    )
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
