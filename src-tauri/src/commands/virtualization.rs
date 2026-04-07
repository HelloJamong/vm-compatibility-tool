/// 가상화 설정 점검 커맨드 — Phase 0 PoC
///
/// Registry + 프로세스(dism, bcdedit) 기반 점검

use crate::models::virtualization::VirtualizationItem;
use crate::services::{process_service, registry_service::windows as reg};

#[tauri::command]
pub async fn get_virtualization_status() -> Result<Vec<VirtualizationItem>, String> {
    tokio::task::spawn_blocking(|| collect_virtualization_status())
        .await
        .map_err(|e| format!("작업 실행 오류: {e}"))?
        .map_err(|e| e.to_string())
}

fn collect_virtualization_status() -> anyhow::Result<Vec<VirtualizationItem>> {
    let mut items = Vec::new();

    // 1. 하드웨어 가상화 (WMI — Phase 1에서 완성)
    items.push(VirtualizationItem::new(
        "하드웨어 가상화",
        "확인 필요",
        "Phase 1에서 WMI Win32_Processor 기반 구현 예정",
        "",
    ));

    // 2. Hyper-V 상태 (DISM)
    check_hyperv_status(&mut items);

    // 3. WSL 상태 (DISM)
    check_wsl_status(&mut items);

    // 4. Hypervisor 시작 유형 (bcdedit)
    check_hypervisor_launch(&mut items);

    // 5. VBS 상태 (Registry)
    check_vbs_status(&mut items);

    Ok(items)
}

fn check_hyperv_status(items: &mut Vec<VirtualizationItem>) {
    let features = [
        ("Microsoft-Hyper-V-All", "Hyper-V (전체)"),
        ("Microsoft-Hyper-V-Hypervisor", "Hyper-V 하이퍼바이저"),
    ];

    for (feature, label) in features {
        let result = process_service::get_feature_state(feature);
        let (status, details, rec) = if !result.success {
            ("확인 불가", "DISM 실행 실패".to_string(), "관리자 권한으로 실행하세요")
        } else if result.stdout.contains("State : Enabled") {
            ("설치됨 (활성)", format!("{} 가 활성화되어 있습니다", feature),
             "VM 사용을 위해 비활성화가 필요합니다")
        } else if result.stdout.contains("State : Disabled") {
            ("설치됨 (비활성)", format!("{} 가 비활성화되어 있습니다", feature), "")
        } else {
            ("미설치", format!("{} 가 설치되어 있지 않습니다", feature), "")
        };

        items.push(VirtualizationItem::new(label, status, &details, rec));
    }
}

fn check_wsl_status(items: &mut Vec<VirtualizationItem>) {
    let features = [
        ("Microsoft-Windows-Subsystem-Linux", "WSL"),
        ("VirtualMachinePlatform", "가상 머신 플랫폼 (WSL2)"),
    ];

    for (feature, label) in features {
        let result = process_service::get_feature_state(feature);
        let (status, details, rec) = if result.stdout.contains("State : Enabled") {
            ("설치됨 (활성)", format!("{} 가 활성화되어 있습니다", label),
             "VM 성능 향상을 위해 비활성화를 권장합니다")
        } else {
            ("비활성 또는 미설치", format!("{} 가 비활성화되어 있습니다", label), "")
        };

        items.push(VirtualizationItem::new(label, status, &details, rec));
    }
}

fn check_hypervisor_launch(items: &mut Vec<VirtualizationItem>) {
    let launch_type = process_service::get_hypervisor_launch_type();
    let is_active = launch_type.to_lowercase() != "off"
        && !launch_type.contains("확인 불가")
        && !launch_type.contains("오류");

    items.push(VirtualizationItem::new(
        "Hypervisor 시작 유형",
        &launch_type,
        &format!("bcdedit hypervisorlaunchtype: {}", launch_type),
        if is_active { "비활성화를 위해 bcdedit /set hypervisorlaunchtype off 실행 필요" } else { "" },
    ));
}

fn check_vbs_status(items: &mut Vec<VirtualizationItem>) {
    let vbs = reg::get_vbs_enabled();
    let hvci = reg::get_hvci_enabled();
    let cred_guard = reg::get_credential_guard_enabled();

    items.push(VirtualizationItem::new(
        "VBS (가상화 기반 보안)",
        if vbs { "활성화됨" } else { "비활성화됨" },
        "SYSTEM\\CurrentControlSet\\Control\\DeviceGuard",
        if vbs { "VM 호환성을 위해 비활성화가 필요합니다" } else { "" },
    ));

    items.push(VirtualizationItem::new(
        "HVCI (코어 격리)",
        if hvci { "활성화됨" } else { "비활성화됨" },
        "DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity",
        if hvci { "비활성화가 필요합니다" } else { "" },
    ));

    items.push(VirtualizationItem::new(
        "CredentialGuard",
        if cred_guard { "활성화됨" } else { "비활성화됨" },
        "DeviceGuard\\Scenarios\\CredentialGuard",
        if cred_guard { "비활성화가 필요합니다" } else { "" },
    ));
}
