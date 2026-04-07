/// 시스템 정보 수집 커맨드 — Phase 0 PoC
///
/// 프론트엔드에서 invoke("get_system_info") 로 호출
/// WMI + Registry 양쪽 모두 검증

use crate::models::system_info::SystemInfoItem;
use crate::services::registry_service::windows as reg;

#[tauri::command]
pub async fn get_system_info() -> Result<Vec<SystemInfoItem>, String> {
    tokio::task::spawn_blocking(|| {
        collect_all_system_info()
    })
    .await
    .map_err(|e| format!("작업 실행 오류: {e}"))?
    .map_err(|e| e.to_string())
}

fn collect_all_system_info() -> anyhow::Result<Vec<SystemInfoItem>> {
    let mut items = Vec::new();

    // 1. OS 정보 (Registry)
    collect_os_info(&mut items);

    // 2. CPU 정보 (WMI)
    collect_cpu_info(&mut items);

    // 3. 메모리 정보 (WMI)
    collect_memory_info(&mut items);

    // 4. 디스크 정보 (WMI)
    collect_disk_info(&mut items);

    // 5. 부팅 시간 (WMI)
    collect_boot_info(&mut items);

    Ok(items)
}

fn collect_os_info(items: &mut Vec<SystemInfoItem>) {
    let info = reg::get_windows_version();
    items.push(SystemInfoItem::new("운영체제", "이름", &info.os_name));
    items.push(SystemInfoItem::new("운영체제", "버전", &info.display_version));
    items.push(SystemInfoItem::new(
        "운영체제",
        "빌드",
        &format!("{}.{}", info.build_number, info.ubr),
    ));
    items.push(SystemInfoItem::new("운영체제", "에디션", &info.product_name));
}

#[cfg(windows)]
fn collect_cpu_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::poc_get_cpu_info() {
        Ok(processors) => {
            if let Some(cpu) = processors.into_iter().next() {
                items.push(SystemInfoItem::new("프로세서", "모델", &cpu.name));
                items.push(SystemInfoItem::new("프로세서", "제조사", &cpu.manufacturer));
                items.push(SystemInfoItem::new(
                    "프로세서", "코어 수",
                    &cpu.number_of_cores.to_string(),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서", "논리 프로세서",
                    &cpu.number_of_logical_processors.to_string(),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서", "최대 클럭",
                    &format!("{} MHz", cpu.max_clock_speed),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서", "하드웨어 가상화",
                    if cpu.virtualization_firmware_enabled { "활성화됨" } else { "비활성화됨" },
                ));
            }
        }
        Err(e) => {
            items.push(SystemInfoItem::error("프로세서", &e.to_string()));
        }
    }
}

#[cfg(not(windows))]
fn collect_cpu_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("프로세서", "Windows 전용 기능"));
}

#[cfg(windows)]
fn collect_memory_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::poc_get_system_info() {
        Ok(systems) => {
            if let Some(sys) = systems.into_iter().next() {
                let total_gb = sys.total_physical_memory as f64 / (1024.0 * 1024.0 * 1024.0);
                items.push(SystemInfoItem::new(
                    "메모리", "총 용량",
                    &format!("{:.1} GB", total_gb),
                ));
                items.push(SystemInfoItem::new("메모리", "제조사", &sys.manufacturer));
            }
        }
        Err(e) => {
            items.push(SystemInfoItem::error("메모리", &e.to_string()));
        }
    }
}

#[cfg(not(windows))]
fn collect_memory_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("메모리", "Windows 전용 기능"));
}

// 디스크/부팅 정보 — Phase 1에서 구현 예정
fn collect_disk_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::new("디스크", "상태", "Phase 1에서 구현 예정"));
}

fn collect_boot_info(items: &mut Vec<SystemInfoItem>) {
    // 빠른 폴백: Environment 기반 (C# 원본과 동일)
    let uptime_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok();

    items.push(SystemInfoItem::new("부팅", "상태", "Phase 1에서 WMI 기반 구현 예정"));
    let _ = uptime_secs; // Phase 1에서 활용
}
