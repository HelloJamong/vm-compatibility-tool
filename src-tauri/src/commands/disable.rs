/// 비활성화 실행 커맨드
use crate::models::virtualization::{DisableGroup, DisableOptions, DisableResult, ProgressEvent};
use crate::services::{
    log_service, process_service, registry_manifest, registry_service::windows as reg,
};
use tauri::{AppHandle, Emitter};

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

/// 비활성화 실행
///
/// - `options`: None이면 전체 실행, Some이면 가상화 점검 결과 기반 선택 실행
#[tauri::command]
pub async fn execute_disable(
    app: AppHandle,
    options: Option<DisableOptions>,
) -> Result<Vec<DisableResult>, String> {
    tokio::task::spawn_blocking(move || {
        run_disable_tasks(&app, options.unwrap_or_else(DisableOptions::all))
    })
    .await
    .map_err(|e| format!("작업 실행 오류: {e}"))?
    .map_err(|e| e.to_string())
}

fn run_disable_tasks(app: &AppHandle, opts: DisableOptions) -> anyhow::Result<Vec<DisableResult>> {
    log_service::init();

    type TaskFn = fn() -> DisableResult;
    let mut tasks: Vec<(&str, TaskFn)> = Vec::new();

    if opts.hyperv {
        tasks.push(("Hyper-V 기능 비활성화", disable_hyperv));
    }
    if opts.wsl {
        tasks.push(("WSL 비활성화", disable_wsl));
    }
    if opts.vbs {
        tasks.push(("VBS 레지스트리 비활성화", disable_vbs));
    }
    if opts.core_isolation {
        tasks.push(("코어 격리 비활성화", disable_core_isolation));
    }

    if tasks.is_empty() {
        return Ok(vec![DisableResult {
            task: "점검 결과".to_string(),
            success: true,
            message: "비활성화가 필요한 항목이 없습니다.".to_string(),
        }]);
    }

    let total = tasks.len() as u32;
    let mut results = Vec::new();

    for (i, (label, task_fn)) in tasks.into_iter().enumerate() {
        let step = i as u32 + 1;

        let _ = app.emit(
            "disable-progress",
            ProgressEvent {
                step,
                total,
                message: label.to_string(),
                success: true,
            },
        );

        let result = task_fn();

        if !result.success {
            let _ = app.emit(
                "disable-progress",
                ProgressEvent {
                    step,
                    total,
                    message: format!("{label} — 일부 실패"),
                    success: false,
                },
            );
            log_service::log_error(label, &result.message);
        }

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
        }
    }

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
    disable_registry_group(DisableGroup::Vbs, "VBS 비활성화")
}

fn disable_core_isolation() -> DisableResult {
    disable_registry_group(DisableGroup::CoreIsolation, "코어 격리 비활성화")
}

fn disable_registry_group(group: DisableGroup, task_name: &str) -> DisableResult {
    let mut messages = Vec::new();
    let mut success = true;

    for entry in registry_manifest::disable_write_entries(group) {
        let target_value = entry.target_value.unwrap_or(0);
        match reg::get_dword(&entry.path, entry.value_name) {
            Some(current_value) if current_value != target_value => {
                match reg::set_dword(&entry.path, entry.value_name, target_value) {
                    Ok(_) => messages.push(format!(
                        "✓ {} — {}\\{}: {} → {}",
                        entry.label, entry.path, entry.value_name, current_value, target_value
                    )),
                    Err(error) => {
                        messages.push(format!(
                            "✗ {} — {}\\{}: {}",
                            entry.label, entry.path, entry.value_name, error
                        ));
                        success = false;
                    }
                }
            }
            Some(current_value) => {
                messages.push(format!(
                    "- {} — {}\\{} = {} (이미 비활성 상태)",
                    entry.label, entry.path, entry.value_name, current_value
                ));
            }
            None => {
                messages.push(format!(
                    "- {} — {}\\{} (값 없음, 생성하지 않음)",
                    entry.label, entry.path, entry.value_name
                ));
            }
        }
    }

    DisableResult {
        task: task_name.to_string(),
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
