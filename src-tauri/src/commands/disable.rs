/// 비활성화 실행 커맨드 — Phase 0 PoC (구조만, 실제 실행은 Phase 2에서 완성)

use crate::models::virtualization::{DisableResult, ProgressEvent};
use crate::services::{process_service, registry_service::windows as reg};
use tauri::{AppHandle, Emitter};

/// VBS 레지스트리 비활성화 대상 (C# DisableVBS() L3199 기반)
const VBS_REGISTRY_KEYS: &[(&str, &str)] = &[
    (r"SYSTEM\CurrentControlSet\Control\DeviceGuard", "EnableVirtualizationBasedSecurity"),
    (r"SYSTEM\CurrentControlSet\Control\DeviceGuard", "RequirePlatformSecurityFeatures"),
    (r"SYSTEM\ControlSet001\Control\DeviceGuard", "EnableVirtualizationBasedSecurity"),
    (r"SYSTEM\ControlSet001\Control\DeviceGuard", "RequirePlatformSecurityFeatures"),
    (r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled"),
    (r"SYSTEM\ControlSet001\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity", "Enabled"),
    (r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard", "Enabled"),
    (r"SYSTEM\ControlSet001\Control\DeviceGuard\Scenarios\CredentialGuard", "Enabled"),
    (r"SYSTEM\CurrentControlSet\Control\Lsa", "LsaCfgFlags"),
    (r"SYSTEM\ControlSet001\Control\Lsa", "LsaCfgFlags"),
];

/// 코어 격리 레지스트리 비활성화 대상 (C# DisableCoreIsolation() L3290 기반)
const CORE_ISOLATION_KEYS: &[(&str, &str)] = &[
    (r"SYSTEM\CurrentControlSet\Control\CI\Config", "VulnerableDriverBlocklistEnable"),
    (r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "EnableVirtualizationBasedSecurity"),
    (r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HypervisorEnforcedCodeIntegrity"),
    (r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard", "HVCIEnabled"),
];

/// Hyper-V 비활성화 대상 Feature 목록 (C# DisableHyperVFeatures() L3040 기반)
const HYPERV_FEATURES: &[&str] = &[
    "Microsoft-Hyper-V-All",
    "Microsoft-Hyper-V",
    "Microsoft-Hyper-V-Tools-All",
    "Microsoft-Hyper-V-Management-PowerShell",
    "Microsoft-Hyper-V-Hypervisor",
    "Microsoft-Hyper-V-Services",
    "Microsoft-Hyper-V-Management-Clients",
];

const WSL_FEATURES: &[&str] = &[
    "Microsoft-Windows-Subsystem-Linux",
    "VirtualMachinePlatform",
];

#[tauri::command]
pub async fn execute_disable(
    app: AppHandle,
    selective: bool,
) -> Result<Vec<DisableResult>, String> {
    tokio::task::spawn_blocking(move || run_disable_tasks(&app, selective))
        .await
        .map_err(|e| format!("작업 실행 오류: {e}"))?
        .map_err(|e| e.to_string())
}

fn run_disable_tasks(app: &AppHandle, _selective: bool) -> anyhow::Result<Vec<DisableResult>> {
    let tasks: Vec<(&str, fn() -> DisableResult)> = vec![
        ("Hyper-V 기능 비활성화", disable_hyperv),
        ("WSL 비활성화", disable_wsl),
        ("VBS 레지스트리 비활성화", disable_vbs),
        ("코어 격리 비활성화", disable_core_isolation),
    ];

    let total = tasks.len() as u32;
    let mut results = Vec::new();

    for (i, (label, task_fn)) in tasks.into_iter().enumerate() {
        let step = i as u32 + 1;

        // 진행 상태 이벤트 전송
        let _ = app.emit(
            "disable-progress",
            ProgressEvent { step, total, message: label.to_string(), success: true },
        );

        let result = task_fn();
        results.push(result);
    }

    Ok(results)
}

fn disable_hyperv() -> DisableResult {
    let mut messages = Vec::new();
    let mut all_success = true;

    for feature in HYPERV_FEATURES {
        let r = process_service::disable_feature(feature);
        if r.success {
            messages.push(format!("✓ {feature}"));
        } else {
            messages.push(format!("- {feature} (이미 비활성화됨)"));
            // exit code != 0 이어도 "이미 비활성화" 상태일 수 있어 실패로 처리 안 함
        }
    }

    // bcdedit hypervisorlaunchtype off
    let bcd = process_service::disable_hypervisor_launch();
    if bcd.success {
        messages.push("✓ hypervisorlaunchtype off".to_string());
    } else {
        messages.push(format!("✗ bcdedit 실패: {}", bcd.stderr));
        all_success = false;
    }

    DisableResult {
        task: "Hyper-V 비활성화".to_string(),
        success: all_success,
        message: messages.join("\n"),
    }
}

fn disable_wsl() -> DisableResult {
    let mut messages = Vec::new();

    for feature in WSL_FEATURES {
        let r = process_service::disable_feature(feature);
        messages.push(if r.success {
            format!("✓ {feature}")
        } else {
            format!("- {feature} (이미 비활성화됨)")
        });
    }

    DisableResult {
        task: "WSL 비활성화".to_string(),
        success: true,
        message: messages.join("\n"),
    }
}

fn disable_vbs() -> DisableResult {
    let mut messages = Vec::new();
    let mut success = true;

    for (path, name) in VBS_REGISTRY_KEYS {
        match reg::set_dword(path, name, 0) {
            Ok(_) => messages.push(format!("✓ {}\\{} = 0", path, name)),
            Err(e) => {
                messages.push(format!("✗ {}\\{}: {}", path, name, e));
                success = false;
            }
        }
    }

    DisableResult {
        task: "VBS 비활성화".to_string(),
        success,
        message: messages.join("\n"),
    }
}

#[tauri::command]
pub fn request_reboot() -> Result<(), String> {
    let result = process_service::schedule_reboot();
    if result.success {
        Ok(())
    } else {
        Err(format!("재부팅 명령 실패: {}", result.stderr))
    }
}

fn disable_core_isolation() -> DisableResult {
    let mut messages = Vec::new();
    let mut success = true;

    for (path, name) in CORE_ISOLATION_KEYS {
        match reg::set_dword(path, name, 0) {
            Ok(_) => messages.push(format!("✓ {}\\{} = 0", path, name)),
            Err(e) => {
                messages.push(format!("✗ {}\\{}: {}", path, name, e));
                success = false;
            }
        }
    }

    DisableResult {
        task: "코어 격리 비활성화".to_string(),
        success,
        message: messages.join("\n"),
    }
}
