/// 시스템 정보 수집 커맨드
///
/// 프론트엔드에서 invoke("get_system_info") 로 호출
/// WMI + Registry 양쪽 수집

use crate::models::system_info::SystemInfoItem;
use crate::services::{event_log_service, registry_service::windows as reg};
use tauri::AppHandle;

#[tauri::command]
pub fn get_app_version() -> String {
    option_env!("TAURI_DISPLAY_VERSION")
        .unwrap_or("dev")
        .to_string()
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn get_system_info() -> Result<Vec<SystemInfoItem>, String> {
    tokio::task::spawn_blocking(collect_all_system_info)
        .await
        .map_err(|e| format!("작업 실행 오류: {e}"))?
        .map_err(|e| e.to_string())
}

fn collect_all_system_info() -> anyhow::Result<Vec<SystemInfoItem>> {
    let mut items = Vec::new();

    collect_os_info(&mut items);
    collect_cpu_info(&mut items);
    collect_memory_info(&mut items);
    collect_disk_info(&mut items);
    collect_boot_info(&mut items);
    collect_motherboard_info(&mut items);
    collect_gpu_info(&mut items);
    collect_power_info(&mut items);
    collect_windows_update_info(&mut items);
    event_log_service::collect_event_log_info(&mut items);

    Ok(items)
}

// ── OS 정보 (Registry) ─────────────────────────────────────────────────────

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
    items.push(SystemInfoItem::new("운영체제", "아키텍처", &info.architecture));
    items.push(SystemInfoItem::new("운영체제", "설치 날짜", &info.install_date));
    items.push(SystemInfoItem::new("운영체제", "설치 언어", &info.install_language));
}

// ── CPU 정보 (WMI) ─────────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_cpu_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::poc_get_cpu_info() {
        Ok(processors) => {
            if let Some(cpu) = processors.into_iter().next() {
                items.push(SystemInfoItem::new("프로세서", "모델", &cpu.name));
                items.push(SystemInfoItem::new("프로세서", "제조사", &cpu.manufacturer));
                items.push(SystemInfoItem::new(
                    "프로세서",
                    "코어 수",
                    &cpu.number_of_cores.to_string(),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서",
                    "논리 프로세서",
                    &cpu.number_of_logical_processors.to_string(),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서",
                    "최대 클럭",
                    &format!("{} MHz", cpu.max_clock_speed),
                ));
                items.push(SystemInfoItem::new(
                    "프로세서",
                    "하드웨어 가상화",
                    if cpu.virtualization_firmware_enabled {
                        "활성화됨"
                    } else {
                        "비활성화됨"
                    },
                ));
            }
        }
        Err(e) => items.push(SystemInfoItem::error("프로세서", &e.to_string())),
    }
}

#[cfg(not(windows))]
fn collect_cpu_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("프로세서", "Windows 전용 기능"));
}

// ── 메모리 정보 (WMI) ──────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_memory_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    // 총 메모리 (Win32_ComputerSystem)
    match wmi::poc_get_system_info() {
        Ok(systems) => {
            if let Some(sys) = systems.into_iter().next() {
                let total_gb = sys.total_physical_memory as f64 / (1024.0 * 1024.0 * 1024.0);
                items.push(SystemInfoItem::new(
                    "메모리",
                    "총 용량",
                    &format!("{:.1} GB", total_gb),
                ));
                items.push(SystemInfoItem::new("메모리", "제조사", &sys.manufacturer));
            }
        }
        Err(e) => items.push(SystemInfoItem::error("메모리", &e.to_string())),
    }

    // 가용 메모리 (Win32_OperatingSystem)
    if let Ok(os_list) = wmi::get_os_info() {
        if let Some(os) = os_list.into_iter().next() {
            let free_mb = os.free_physical_memory / 1024;
            items.push(SystemInfoItem::new(
                "메모리",
                "가용 용량",
                &format!("{} MB", free_mb),
            ));
        }
    }
}

#[cfg(not(windows))]
fn collect_memory_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("메모리", "Windows 전용 기능"));
}

// ── 디스크 정보 (WMI) ──────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_disk_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::{disk_service, wmi_service::windows as wmi};

    let drives = match wmi::get_disk_drives() {
        Ok(d) => d,
        Err(e) => {
            items.push(SystemInfoItem::error("디스크", &e.to_string()));
            return;
        }
    };

    // MSFT_PhysicalDisk — 미디어 타입(SSD/HDD) 판별용 (실패해도 무시)
    let physical_disks = wmi::get_msft_physical_disks().unwrap_or_default();

    if drives.is_empty() {
        items.push(SystemInfoItem::new("디스크", "상태", "디스크를 찾을 수 없습니다"));
        return;
    }

    for (i, drive) in drives.iter().enumerate() {
        let label = if drives.len() > 1 {
            format!("디스크 {}", i + 1)
        } else {
            "디스크".to_string()
        };

        items.push(SystemInfoItem::new(&label, "모델", &drive.model));

        if let Some(size) = drive.size {
            items.push(SystemInfoItem::new(&label, "용량", &format_disk_size(size)));
        }

        if let Some(iface) = &drive.interface_type {
            if !iface.is_empty() {
                items.push(SystemInfoItem::new(&label, "인터페이스", iface));
            }
        }

        // USB 인터페이스는 MediaType/모델명 판별 전에 먼저 확정
        let disk_type = if drive.interface_type.as_deref() == Some("USB") {
            disk_service::DiskType::Usb
        } else {
            // MSFT_PhysicalDisk 인덱스 대응 → MediaType=0(Unknown)이면 모델명 키워드 폴백
            let from_wmi = physical_disks
                .get(i)
                .and_then(|pd| {
                    pd.media_type
                        .map(|mt| disk_service::media_type_to_disk_type(mt, pd.bus_type))
                });
            match from_wmi {
                Some(t) if t != disk_service::DiskType::Unknown => t,
                _ => disk_service::detect_from_model_name(&drive.model),
            }
        };

        items.push(SystemInfoItem::new(&label, "타입", &disk_type.to_string()));
    }
}

#[cfg(not(windows))]
fn collect_disk_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("디스크", "Windows 전용 기능"));
}

// ── 부팅 시간 (WMI) ────────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_boot_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::get_os_info() {
        Ok(os_list) => {
            if let Some(os) = os_list.into_iter().next() {
                items.push(SystemInfoItem::new(
                    "부팅",
                    "마지막 부팅",
                    &parse_wmi_datetime(&os.last_boot_up_time),
                ));
                items.push(SystemInfoItem::new(
                    "부팅",
                    "가동 시간",
                    &compute_uptime(&os.last_boot_up_time),
                ));
            }
        }
        Err(e) => items.push(SystemInfoItem::error("부팅", &e.to_string())),
    }
}

#[cfg(not(windows))]
fn collect_boot_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("부팅", "Windows 전용 기능"));
}

// ── 메인보드 정보 (WMI) ────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_motherboard_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::get_baseboard_info() {
        Ok(boards) => {
            if let Some(board) = boards.into_iter().next() {
                items.push(SystemInfoItem::new("메인보드", "제조사", &board.manufacturer));
                items.push(SystemInfoItem::new("메인보드", "모델", &board.product));
                if let Some(sn) = &board.serial_number {
                    let sn = sn.trim();
                    if !sn.is_empty() && sn != "Default string" && sn != "To Be Filled By O.E.M." {
                        items.push(SystemInfoItem::new("메인보드", "시리얼 번호", sn));
                    }
                }
            }
        }
        Err(e) => items.push(SystemInfoItem::error("메인보드", &e.to_string())),
    }
}

#[cfg(not(windows))]
fn collect_motherboard_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("메인보드", "Windows 전용 기능"));
}

// ── GPU 정보 (WMI) ─────────────────────────────────────────────────────────

#[cfg(windows)]
fn collect_gpu_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::get_video_controllers() {
        Ok(gpus) => {
            if gpus.is_empty() {
                items.push(SystemInfoItem::new("GPU", "상태", "GPU를 찾을 수 없습니다"));
                return;
            }
            for (i, gpu) in gpus.iter().enumerate() {
                let label = if gpus.len() > 1 {
                    format!("GPU {}", i + 1)
                } else {
                    "GPU".to_string()
                };

                items.push(SystemInfoItem::new(&label, "모델", &gpu.name));

                if let Some(ram) = gpu.adapter_ram {
                    if ram > 0 {
                        items.push(SystemInfoItem::new(&label, "VRAM", &format_vram(ram)));
                    }
                }

                if let Some(ver) = &gpu.driver_version {
                    if !ver.is_empty() {
                        items.push(SystemInfoItem::new(&label, "드라이버 버전", ver));
                    }
                }
            }
        }
        Err(e) => items.push(SystemInfoItem::error("GPU", &e.to_string())),
    }
}

#[cfg(not(windows))]
fn collect_gpu_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("GPU", "Windows 전용 기능"));
}

// ── 전원 관리 정보 (WMI) ───────────────────────────────────────────────────

#[cfg(windows)]
fn collect_power_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::wmi_service::windows as wmi;

    match wmi::get_power_plans() {
        Ok(plans) => {
            if plans.is_empty() {
                items.push(SystemInfoItem::new("전원", "상태", "전원 계획을 찾을 수 없습니다"));
                return;
            }

            // 현재 활성 전원 계획
            if let Some(active) = plans.iter().find(|p| p.is_active == Some(true)) {
                items.push(SystemInfoItem::new(
                    "전원",
                    "현재 전원 관리 옵션",
                    &active.element_name,
                ));
            }

            // 전체 목록
            let all: Vec<String> = plans
                .iter()
                .map(|p| {
                    if p.is_active == Some(true) {
                        format!("{} (현재)", p.element_name)
                    } else {
                        p.element_name.clone()
                    }
                })
                .collect();
            items.push(SystemInfoItem::new("전원", "등록된 전원 계획", &all.join(", ")));
        }
        Err(e) => items.push(SystemInfoItem::error("전원", &e.to_string())),
    }
}

#[cfg(not(windows))]
fn collect_power_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("전원", "Windows 전용 기능"));
}

// ── Windows 업데이트 이력 (PowerShell WUA COM) ─────────────────────────────

#[cfg(windows)]
fn collect_windows_update_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::process_service;

    let script = r#"
$cutoff = (Get-Date).AddDays(-90)
try {
    $session  = New-Object -ComObject Microsoft.Update.Session
    $searcher = $session.CreateUpdateSearcher()
    $count    = $searcher.GetTotalHistoryCount()
    if ($count -eq 0) { exit 0 }
    $limit    = [Math]::Min($count, 300)
    $history  = $searcher.QueryHistory(0, $limit)
    $results  = $history | Where-Object {
        $_.Date -ge $cutoff -and $_.ResultCode -eq 2
    } | ForEach-Object {
        if ($_.Title -match '(KB\d+)') {
            "$($_.Date.ToString('yyyy-MM-dd'))|$($Matches[1])"
        }
    } | Where-Object { $_ } | Select-Object -Unique
    if ($results) { $results | Write-Output }
} catch {
    Write-Error $_.Exception.Message
}
"#;

    let result = process_service::run_powershell(script);

    if !result.success && result.stdout.trim().is_empty() {
        let msg = result.stderr.trim();
        items.push(SystemInfoItem::error(
            "Windows 업데이트",
            if msg.is_empty() { "수집 실패" } else { msg },
        ));
        return;
    }

    let mut count = 0u32;
    for line in result.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '|');
        if let (Some(date), Some(kb)) = (parts.next(), parts.next()) {
            let date = date.trim();
            let kb = kb.trim();
            if !kb.is_empty() && !date.is_empty() {
                items.push(SystemInfoItem::new("Windows 업데이트", kb, date));
                count += 1;
            }
        }
    }

    if count == 0 {
        items.push(SystemInfoItem::new(
            "Windows 업데이트",
            "최근 3개월",
            "업데이트 기록 없음",
        ));
    }
}

#[cfg(not(windows))]
fn collect_windows_update_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("Windows 업데이트", "Windows 전용 기능"));
}

// ── 유틸 함수 ──────────────────────────────────────────────────────────────

/// WMI datetime 문자열 파싱 (형식: YYYYMMDDHHMMSS.ffffff±TZO)
fn parse_wmi_datetime(s: &str) -> String {
    if s.len() < 14 {
        return s.to_string();
    }
    chrono::NaiveDateTime::parse_from_str(&s[..14], "%Y%m%d%H%M%S")
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|_| s.to_string())
}

/// WMI LastBootUpTime 기준 가동 시간 계산
fn compute_uptime(boot_wmi: &str) -> String {
    if boot_wmi.len() < 14 {
        return "알 수 없음".to_string();
    }
    let Ok(boot_dt) = chrono::NaiveDateTime::parse_from_str(&boot_wmi[..14], "%Y%m%d%H%M%S")
    else {
        return "알 수 없음".to_string();
    };

    let now = chrono::Local::now().naive_local();
    let duration = now.signed_duration_since(boot_dt);
    let days = duration.num_days();
    let hours = duration.num_hours().abs() % 24;
    let minutes = duration.num_minutes().abs() % 60;
    format!("{}일 {}시간 {}분", days, hours, minutes)
}

/// 바이트 → GB/TB 문자열 변환 (디스크 용량)
fn format_disk_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.1} TB", bytes as f64 / 1_000_000_000_000.0)
    } else {
        format!("{:.0} GB", bytes as f64 / 1_000_000_000.0)
    }
}

/// 바이트 → MB/GB 문자열 변환 (VRAM)
fn format_vram(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.0} GB", bytes as f64 / 1_073_741_824.0)
    } else {
        format!("{:.0} MB", bytes as f64 / 1_048_576.0)
    }
}
