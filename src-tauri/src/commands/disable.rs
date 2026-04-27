/// 비활성화 실행 커맨드
use crate::models::virtualization::{
    DisableGroup, DisableOptions, DisableOutput, DisableResult, ProgressEvent,
};
use crate::services::{
    log_service, process_service,
    registry_manifest::{self, ResolvedRegistryManifestEntry},
    registry_service::windows as reg,
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

#[derive(Debug, Clone)]
enum ChangeTarget {
    Feature(&'static str),
    HypervisorLaunchType,
    VsmLaunchType,
    Registry {
        path: String,
        value_name: &'static str,
    },
}

#[derive(Debug, Clone)]
struct PlannedDisableChange {
    group: String,
    item: String,
    target: String,
    target_kind: ChangeTarget,
    before: String,
}

/// 비활성화 실행
///
/// - `options`: None이면 전체 실행, Some이면 가상화 점검 결과 기반 선택 실행
#[tauri::command]
pub async fn execute_disable(
    app: AppHandle,
    options: Option<DisableOptions>,
) -> Result<DisableOutput, String> {
    tokio::task::spawn_blocking(move || {
        run_disable_tasks(&app, options.unwrap_or_else(DisableOptions::all))
    })
    .await
    .map_err(|e| format!("작업 실행 오류: {e}"))?
    .map_err(|e| e.to_string())
}

fn run_disable_tasks(app: &AppHandle, opts: DisableOptions) -> anyhow::Result<DisableOutput> {
    log_service::init();

    // 레지스트리 수정 전 원본값 백업 수집
    let backup_entries = collect_registry_backup(&opts);

    type TaskFn = Box<dyn Fn() -> DisableResult>;
    let mut tasks: Vec<(String, TaskFn)> = Vec::new();

    if opts.hyperv {
        tasks.push((
            "Hyper-V 기능 비활성화".to_string(),
            Box::new(disable_hyperv),
        ));
    }
    if opts.wsl {
        tasks.push(("WSL 비활성화".to_string(), Box::new(disable_wsl)));
    }
    if opts.vbs {
        tasks.push(("VBS 레지스트리 비활성화".to_string(), Box::new(disable_vbs)));
    }
    if opts.core_isolation {
        tasks.push((
            "코어 격리 비활성화".to_string(),
            Box::new(disable_core_isolation),
        ));
    }
    if !opts.optional_registry_ids.is_empty() {
        let optional_ids = opts.optional_registry_ids.clone();
        tasks.push((
            "선택한 추가 레지스트리 조치".to_string(),
            Box::new(move || disable_optional_registry_entries(&optional_ids)),
        ));
    }

    if tasks.is_empty() {
        let result = DisableResult {
            task: "점검 결과".to_string(),
            success: true,
            message: "비활성화가 필요한 항목이 없습니다.".to_string(),
        };
        return Ok(DisableOutput {
            results: vec![result],
            log_path: None,
            backup_path: None,
            change_csv_path: None,
        });
    }

    let planned_changes = collect_planned_disable_changes(&opts);

    let total = tasks.len() as u32;
    let mut results = Vec::new();
    let mut log_lines: Vec<String> = Vec::new();

    log_lines.push("▶ 비활성화 작업 시작".to_string());

    for (i, (label, task_fn)) in tasks.into_iter().enumerate() {
        let step = i as u32 + 1;

        let _ = app.emit(
            "disable-progress",
            ProgressEvent {
                step,
                total,
                message: label.clone(),
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
            log_service::log_error(&label, &result.message);
        }

        log_lines.push(String::new());
        log_lines.push(format!(
            "{} {}",
            if result.success { "✅" } else { "⚠️" },
            result.task
        ));
        for line in result.message.lines() {
            log_lines.push(format!("  {line}"));
        }

        results.push(result);
    }

    log_lines.push(String::new());
    log_lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
    log_lines.push("모든 작업 완료".to_string());

    // 운영 로그 + 백업 파일 저장
    let (log_path, backup_path) = match log_service::save_operation_log(&log_lines, &backup_entries)
    {
        Some((lp, bp)) => (Some(lp), Some(bp)),
        None => (None, None),
    };
    let change_entries = collect_disable_change_results(planned_changes);
    let change_csv_path = log_service::save_disable_change_csv(&change_entries);

    Ok(DisableOutput {
        results,
        log_path,
        backup_path,
        change_csv_path,
    })
}

fn collect_planned_disable_changes(opts: &DisableOptions) -> Vec<PlannedDisableChange> {
    let mut changes = Vec::new();

    if opts.hyperv {
        for feature in HYPERV_FEATURES {
            push_change(
                &mut changes,
                "Hyper-V",
                feature,
                &format!("DISM Feature: {feature}"),
                ChangeTarget::Feature(feature),
            );
        }
        push_change(
            &mut changes,
            "Hyper-V",
            "Hypervisor 시작 유형",
            "bcdedit hypervisorlaunchtype",
            ChangeTarget::HypervisorLaunchType,
        );
        push_change(
            &mut changes,
            "Hyper-V",
            "VSM 시작 유형",
            "bcdedit vsmlaunchtype",
            ChangeTarget::VsmLaunchType,
        );
    }

    if opts.wsl {
        for feature in WSL_FEATURES {
            push_change(
                &mut changes,
                "WSL",
                feature,
                &format!("DISM Feature: {feature}"),
                ChangeTarget::Feature(feature),
            );
        }
    }

    if opts.vbs {
        append_registry_changes(
            &mut changes,
            "VBS",
            registry_manifest::disable_write_entries(DisableGroup::Vbs),
        );
    }

    if opts.core_isolation {
        append_registry_changes(
            &mut changes,
            "코어 격리",
            registry_manifest::disable_write_entries(DisableGroup::CoreIsolation),
        );
    }

    if !opts.optional_registry_ids.is_empty() {
        append_registry_changes(
            &mut changes,
            "추가 레지스트리 조치",
            registry_manifest::selected_optional_entries(&opts.optional_registry_ids),
        );
    }

    changes
}

fn push_change(
    changes: &mut Vec<PlannedDisableChange>,
    group: &str,
    item: &str,
    target: &str,
    target_kind: ChangeTarget,
) {
    let before = read_change_target_value(&target_kind);
    changes.push(PlannedDisableChange {
        group: group.to_string(),
        item: item.to_string(),
        target: target.to_string(),
        target_kind,
        before,
    });
}

fn append_registry_changes(
    changes: &mut Vec<PlannedDisableChange>,
    group: &str,
    manifest_entries: Vec<ResolvedRegistryManifestEntry>,
) {
    for entry in manifest_entries {
        push_change(
            changes,
            group,
            entry.label,
            &format!(r"HKLM\{}\{}", entry.path, entry.value_name),
            ChangeTarget::Registry {
                path: entry.path,
                value_name: entry.value_name,
            },
        );
    }
}

fn collect_disable_change_results(
    planned_changes: Vec<PlannedDisableChange>,
) -> Vec<log_service::DisableChangeEntry> {
    planned_changes
        .into_iter()
        .map(|change| {
            let after = read_change_target_value(&change.target_kind);
            let (result, message) = compare_change_values(&change.before, &after);
            log_service::DisableChangeEntry {
                group: change.group,
                item: change.item,
                target: change.target,
                before: change.before,
                after,
                result,
                message,
            }
        })
        .collect()
}

fn compare_change_values(before: &str, after: &str) -> (String, String) {
    if before == after {
        (
            "변경 없음".to_string(),
            "이미 목표 상태였거나 조치가 값을 변경하지 않았습니다.".to_string(),
        )
    } else if after.starts_with("확인 불가") || after.starts_with("오류") {
        (
            "확인 불가".to_string(),
            "조치 후 상태를 확인하지 못했습니다.".to_string(),
        )
    } else {
        ("변경됨".to_string(), format!("{before} → {after}"))
    }
}

fn read_change_target_value(target: &ChangeTarget) -> String {
    match target {
        ChangeTarget::Feature(feature) => get_feature_state_display(feature),
        ChangeTarget::HypervisorLaunchType => process_service::get_hypervisor_launch_type(),
        ChangeTarget::VsmLaunchType => process_service::get_vsm_launch_type(),
        ChangeTarget::Registry { path, value_name } => registry_value_display(path, value_name),
    }
}

fn get_feature_state_display(feature: &str) -> String {
    let result = process_service::get_feature_state(feature);
    if !result.success {
        let message = if result.stderr.trim().is_empty() {
            result.stdout.trim()
        } else {
            result.stderr.trim()
        };
        return if message.is_empty() {
            "확인 불가".to_string()
        } else {
            format!("확인 불가: {message}")
        };
    }

    parse_dism_feature_state(&result.stdout).unwrap_or_else(|| "확인 불가".to_string())
}

fn parse_dism_feature_state(stdout: &str) -> Option<String> {
    stdout.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("State") {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

fn registry_value_display(path: &str, value_name: &str) -> String {
    reg::get_dword(path, value_name)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<미설정>".to_string())
}

/// 레지스트리 수정 대상 항목의 현재 값을 수정 전에 수집
fn collect_registry_backup(opts: &DisableOptions) -> Vec<log_service::RegistryBackupEntry> {
    let mut entries = Vec::new();

    let groups: &[(bool, DisableGroup)] = &[
        (opts.vbs, DisableGroup::Vbs),
        (opts.core_isolation, DisableGroup::CoreIsolation),
    ];

    for (enabled, group) in groups {
        if !enabled {
            continue;
        }
        append_registry_backup_entries(
            &mut entries,
            registry_manifest::disable_write_entries(*group),
        );
    }

    if !opts.optional_registry_ids.is_empty() {
        append_registry_backup_entries(
            &mut entries,
            registry_manifest::selected_optional_entries(&opts.optional_registry_ids),
        );
    }

    entries
}

fn append_registry_backup_entries(
    backup_entries: &mut Vec<log_service::RegistryBackupEntry>,
    manifest_entries: Vec<ResolvedRegistryManifestEntry>,
) {
    for manifest_entry in manifest_entries {
        let value = reg::get_dword(&manifest_entry.path, manifest_entry.value_name);
        backup_entries.push(log_service::RegistryBackupEntry {
            path: manifest_entry.path.clone(),
            value_name: manifest_entry.value_name.to_string(),
            value,
        });
    }
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

    let vsm = process_service::disable_vsm_launch();
    if vsm.success {
        messages.push("✓ vsmlaunchtype off".to_string());
    } else {
        messages.push(format!("- vsmlaunchtype: 건너뜀 ({})", vsm.stderr.trim()));
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
    apply_registry_entries(task_name, registry_manifest::disable_write_entries(group))
}

fn disable_optional_registry_entries(ids: &[String]) -> DisableResult {
    apply_registry_entries(
        "선택한 추가 레지스트리 조치",
        registry_manifest::selected_optional_entries(ids),
    )
}

fn apply_registry_entries(
    task_name: &str,
    manifest_entries: Vec<ResolvedRegistryManifestEntry>,
) -> DisableResult {
    let mut messages = Vec::new();
    let mut success = true;

    if manifest_entries.is_empty() {
        messages.push("선택된 추가 레지스트리 조치 항목이 없습니다.".to_string());
    }

    for entry in manifest_entries {
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
