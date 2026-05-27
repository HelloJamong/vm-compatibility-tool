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
    collect_security_hook_info(&mut items);
    event_log_service::collect_event_log_info(&mut items);

    Ok(items)
}

// ── OS 정보 (Registry) ─────────────────────────────────────────────────────

fn collect_os_info(items: &mut Vec<SystemInfoItem>) {
    let info = reg::get_windows_version();
    items.push(SystemInfoItem::new("운영체제", "이름", &info.os_name));
    items.push(SystemInfoItem::new(
        "운영체제",
        "버전",
        &info.display_version,
    ));
    items.push(SystemInfoItem::new(
        "운영체제",
        "빌드",
        &format!("{}.{}", info.build_number, info.ubr),
    ));
    items.push(SystemInfoItem::new(
        "운영체제",
        "에디션",
        &info.product_name,
    ));
    items.push(SystemInfoItem::new(
        "운영체제",
        "아키텍처",
        &info.architecture,
    ));
    items.push(SystemInfoItem::new(
        "운영체제",
        "설치 날짜",
        &info.install_date,
    ));
    items.push(SystemInfoItem::new(
        "운영체제",
        "설치 언어",
        &info.install_language,
    ));
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
        items.push(SystemInfoItem::new(
            "디스크",
            "상태",
            "디스크를 찾을 수 없습니다",
        ));
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
            let from_wmi = physical_disks.get(i).and_then(|pd| {
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
                items.push(SystemInfoItem::new(
                    "메인보드",
                    "제조사",
                    &board.manufacturer,
                ));
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
                items.push(SystemInfoItem::new(
                    "전원",
                    "상태",
                    "전원 계획을 찾을 수 없습니다",
                ));
            } else {
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
                items.push(SystemInfoItem::new(
                    "전원",
                    "등록된 전원 계획",
                    &all.join(", "),
                ));
            }
        }
        Err(e) => items.push(SystemInfoItem::error("전원", &e.to_string())),
    }

    collect_power_reference_info(items);
}

#[cfg(not(windows))]
fn collect_power_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("전원", "Windows 전용 기능"));
}

#[cfg(windows)]
fn collect_power_reference_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::process_service;

    let script = r#"
function Write-Info {
    param(
        [string]$Category,
        [string]$Item,
        [string]$Value
    )

    $safeCategory = ($Category -replace "(`r|`n|`t)+", " ").Trim()
    $safeItem = ($Item -replace "(`r|`n|`t)+", " ").Trim()
    $safeValue = ($Value -replace "(`r|`n|`t)+", " ").Trim()
    "$safeCategory`t$safeItem`t$safeValue" | Write-Output
}

function Format-Seconds {
    param([object]$Value)

    if ($null -eq $Value) { return "확인 불가" }
    $seconds = [int64]$Value
    if ($seconds -eq 0) { return "사용 안 함" }
    if (($seconds % 3600) -eq 0) {
        return "$([int64]($seconds / 3600))시간 ($seconds초)"
    }
    if (($seconds % 60) -eq 0) {
        return "$([int64]($seconds / 60))분 ($seconds초)"
    }
    return "$seconds초"
}

function Format-OnOff {
    param([object]$Value)

    if ($null -eq $Value) { return "확인 불가" }
    if ([int64]$Value -eq 0) { return "사용 안 함" }
    return "사용"
}

function Format-PciExpressAspm {
    param([object]$Value)

    if ($null -eq $Value) { return "확인 불가" }
    switch ([int64]$Value) {
        0 { "끔" }
        1 { "보통 절전" }
        2 { "최대 절전" }
        default { "알 수 없는 값: $Value" }
    }
}

function Format-AcDc {
    param(
        [object]$Setting,
        [scriptblock]$Formatter
    )

    if ($null -eq $Setting) { return "확인 불가" }
    $ac = & $Formatter $Setting.AC
    $dc = & $Formatter $Setting.DC
    return "AC: $ac / DC: $dc"
}

function Get-ActivePowerSchemeGuid {
    try {
        $line = powercfg /getactivescheme 2>$null | Select-Object -First 1
        if ($line -match "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})") {
            return $Matches[1]
        }
    } catch {}

    try {
        return (Get-ItemProperty -LiteralPath "HKLM:\SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes" -Name ActivePowerScheme -ErrorAction Stop).ActivePowerScheme
    } catch {
        return $null
    }
}

function Get-PowerSettingValue {
    param(
        [string]$SchemeGuid,
        [string]$SubgroupGuid,
        [string]$SettingGuid
    )

    if ([string]::IsNullOrWhiteSpace($SchemeGuid)) { return $null }
    $path = "HKLM:\SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes\$SchemeGuid\$SubgroupGuid\$SettingGuid"
    try {
        $prop = Get-ItemProperty -LiteralPath $path -ErrorAction Stop
        [pscustomobject]@{
            AC = $prop.ACSettingIndex
            DC = $prop.DCSettingIndex
        }
    } catch {
        $null
    }
}

$scheme = Get-ActivePowerSchemeGuid
if ($scheme) {
    Write-Info "전원 참고" "활성 전원 계획 GUID" $scheme

    $subSleep = "238c9fa8-0aad-41ed-83f4-97be242c8f20"
    $standbyIdle = "29f6c1db-86da-48c5-9fdb-f2b67b1f44da"
    $hibernateIdle = "9d7815a6-7ee4-497e-8888-515a05f02364"
    $hybridSleep = "94ac6d29-73ce-41a6-809f-6363ba21b47e"
    $subUsb = "2a737441-1930-4402-8d77-b2bebba308a3"
    $usbSelective = "48e6b7a6-50f5-4782-a5d4-53bb8f07e226"
    $subPciExpress = "501a4d13-42af-4429-9fd1-a8218c268e20"
    $aspm = "ee12f906-d277-404b-b6da-e5fa1a576df5"

    Write-Info "전원 참고" "절전 모드 전환 시간" (Format-AcDc (Get-PowerSettingValue $scheme $subSleep $standbyIdle) ${function:Format-Seconds})
    Write-Info "전원 참고" "최대 절전 모드 전환 시간" (Format-AcDc (Get-PowerSettingValue $scheme $subSleep $hibernateIdle) ${function:Format-Seconds})
    Write-Info "전원 참고" "하이브리드 절전" (Format-AcDc (Get-PowerSettingValue $scheme $subSleep $hybridSleep) ${function:Format-OnOff})
    Write-Info "전원 참고" "USB 선택적 절전" (Format-AcDc (Get-PowerSettingValue $scheme $subUsb $usbSelective) ${function:Format-OnOff})
    Write-Info "전원 참고" "PCI Express Link State Power Management" (Format-AcDc (Get-PowerSettingValue $scheme $subPciExpress $aspm) ${function:Format-PciExpressAspm})
} else {
    Write-Info "전원 참고" "활성 전원 계획 GUID" "확인 불가"
}

try {
    $power = Get-ItemProperty -LiteralPath "HKLM:\SYSTEM\CurrentControlSet\Control\Power" -ErrorAction Stop
    $hibernateEnabled = if ($null -ne $power.HibernateEnabled) { $power.HibernateEnabled } elseif ($null -ne $power.HibernateEnabledDefault) { $power.HibernateEnabledDefault } else { $null }
    Write-Info "전원 참고" "최대 절전 모드 활성화 상태" (Format-OnOff $hibernateEnabled)
} catch {
    Write-Info "전원 참고" "최대 절전 모드 활성화 상태" "확인 불가: $($_.Exception.Message)"
}

try {
    $powercfgLines = @(powercfg /a 2>$null)
    $section = ""
    $script:currentUnavailableState = $null
    $script:currentUnavailableReasons = New-Object System.Collections.Generic.List[string]

    function Flush-UnavailableState {
        if (-not [string]::IsNullOrWhiteSpace($script:currentUnavailableState)) {
            $reasonText = if ($script:currentUnavailableReasons.Count -gt 0) {
                ($script:currentUnavailableReasons | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join " / "
            } else {
                "사유 확인 불가"
            }
            Write-Info "전원 참고" "절전 상태 - $script:currentUnavailableState" "사용 불가: $reasonText"
        }

        $script:currentUnavailableState = $null
        $script:currentUnavailableReasons.Clear()
    }

    function Test-PowerStateLine {
        param([string]$Text)
        return $Text -match "^(대기 모드|최대 절전 모드|하이브리드 절전 모드|빠른 시작|Standby|Hibernate|Hybrid Sleep|Fast Startup)"
    }

    foreach ($line in $powercfgLines) {
        $text = ([string]$line).Trim()
        if ([string]::IsNullOrWhiteSpace($text)) { continue }

        if ($text -match "사용할 수 없습니다|not available on this system") {
            Flush-UnavailableState
            $section = "unavailable"
            continue
        }

        if ($text -match "사용할 수 있습니다|available on this system") {
            Flush-UnavailableState
            $section = "available"
            continue
        }

        if ($section -eq "available") {
            if (Test-PowerStateLine $text) {
                Write-Info "전원 참고" "절전 상태 - $text" "사용 가능"
            }
            continue
        }

        if ($section -eq "unavailable") {
            if (Test-PowerStateLine $text) {
                Flush-UnavailableState
                $script:currentUnavailableState = $text
            } elseif (-not [string]::IsNullOrWhiteSpace($script:currentUnavailableState)) {
                $script:currentUnavailableReasons.Add($text)
            }
        }
    }

    Flush-UnavailableState
} catch {}

try {
    $pnpNames = @{}
    Get-CimInstance -ClassName Win32_PnPEntity -ErrorAction Stop |
        Where-Object { $_.PNPDeviceID } |
        ForEach-Object {
            $pnpNames[$_.PNPDeviceID.ToUpperInvariant()] = $_.Name
        }

    $devicePowerEntries = @(Get-CimInstance -Namespace root\wmi -ClassName MSPower_DeviceEnable -ErrorAction Stop)
    $allowCount = 0
    $denyCount = 0

    foreach ($entry in $devicePowerEntries) {
        $instanceId = ([string]$entry.InstanceName) -replace "_\d+$", ""
        $lookupKey = $instanceId.ToUpperInvariant()
        $name = $pnpNames[$lookupKey]
        if ([string]::IsNullOrWhiteSpace($name)) { $name = $instanceId }

        if ($entry.Enable) {
            $allowCount += 1
            $state = "허용"
        } else {
            $denyCount += 1
            $state = "허용 안 함"
        }

        Write-Info "전원 관리 장치" $name "전원을 절약하기 위해 컴퓨터가 이 장치를 끌 수 있음: $state (InstanceId: $instanceId)"
    }

    Write-Info "전원 참고" "장치 전원 끄기 허용 요약" "허용: ${allowCount}개 / 허용 안 함: ${denyCount}개"
} catch {
    Write-Info "전원 관리 장치" "전원을 절약하기 위해 컴퓨터가 이 장치를 끌 수 있음" "확인 불가: $($_.Exception.Message)"
}
"#;

    let result = process_service::run_powershell(script);
    if !result.success && result.stdout.trim().is_empty() {
        let msg = result.stderr.trim();
        items.push(SystemInfoItem::error(
            "전원 참고",
            if msg.is_empty() {
                "전원 참고 정보 수집 실패"
            } else {
                msg
            },
        ));
        return;
    }

    let mut count = 0u32;
    for line in result.stdout.lines() {
        if let Some((category, item, value)) = parse_power_reference_line(line) {
            items.push(SystemInfoItem::new(category, item, value));
            count += 1;
        }
    }

    if count == 0 {
        items.push(SystemInfoItem::new(
            "전원 참고",
            "상태",
            "추가 전원 참고 정보 없음",
        ));
    }
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
    items.push(SystemInfoItem::error(
        "Windows 업데이트",
        "Windows 전용 기능",
    ));
}

// ── 보안/DRM 후킹 모듈 호환성 점검 ────────────────────────────────────────

#[cfg(windows)]
fn collect_security_hook_info(items: &mut Vec<SystemInfoItem>) {
    use crate::services::process_service;

    let script = r#"
$paths = @(
    "$env:WINDIR\System32\f_im.dll",
    "$env:WINDIR\SysWOW64\f_im.dll"
)

foreach ($path in $paths) {
    if (-not (Test-Path -LiteralPath $path)) {
        continue
    }

    try {
        $file = Get-Item -LiteralPath $path
        $version = $file.VersionInfo
        $signature = Get-AuthenticodeSignature -LiteralPath $path

        $company = if ($version.CompanyName) { $version.CompanyName } else { "" }
        $product = if ($version.ProductName) { $version.ProductName } else { "" }
        $fileVersion = if ($version.FileVersion) { $version.FileVersion } else { "" }
        $sigStatus = if ($signature.Status) { $signature.Status.ToString() } else { "Unknown" }
        $signer = if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { "" }

        "$path`t$company`t$product`t$fileVersion`t$sigStatus`t$signer" | Write-Output
    } catch {
        "$path`t`t`t`tCheckFailed`t$($_.Exception.Message)" | Write-Output
    }
}
"#;

    let result = process_service::run_powershell(script);
    if !result.success && result.stdout.trim().is_empty() {
        let msg = result.stderr.trim();
        items.push(SystemInfoItem::error(
            "보안 모듈",
            if msg.is_empty() {
                "f_im.dll 점검 실패"
            } else {
                msg
            },
        ));
        return;
    }

    let mut found = false;
    for line in result.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let path = parts.next().unwrap_or("").trim();
        let company = parts.next().unwrap_or("").trim();
        let product = parts.next().unwrap_or("").trim();
        let file_version = parts.next().unwrap_or("").trim();
        let signature_status = parts.next().unwrap_or("").trim();
        let signer = parts.next().unwrap_or("").trim();

        found = true;
        let location = if path.to_ascii_lowercase().contains("\\syswow64\\") {
            "SysWOW64"
        } else {
            "System32"
        };
        let mut details = Vec::new();
        if !company.is_empty() {
            details.push(format!("회사: {company}"));
        }
        if !product.is_empty() {
            details.push(format!("제품: {product}"));
        }
        if !file_version.is_empty() {
            details.push(format!("버전: {file_version}"));
        }
        details.push(format!(
            "서명: {}",
            if signature_status.is_empty() {
                "Unknown"
            } else {
                signature_status
            }
        ));
        if !signer.is_empty() {
            details.push(format!("서명자: {signer}"));
        }

        items.push(SystemInfoItem::new(
            "보안 모듈",
            &format!("f_im.dll ({location})"),
            &details.join(" / "),
        ));

        if !matches!(signature_status, "Valid" | "") {
            items.push(SystemInfoItem::new(
                "보안 모듈 경고",
                "f_im.dll",
                "Fasoo DRM 계열 전역 후킹 모듈의 서명/상태 이상이 감지되었습니다. 실행 시 0xc0000428 Bad Image 오류가 뜨면 해당 보안 프로그램을 최신 버전으로 재설치하거나 제거 후 재부팅하세요.",
            ));
        }
    }

    if !found {
        items.push(SystemInfoItem::new("보안 모듈", "f_im.dll", "미감지"));
    }
}

#[cfg(not(windows))]
fn collect_security_hook_info(items: &mut Vec<SystemInfoItem>) {
    items.push(SystemInfoItem::error("보안 모듈", "Windows 전용 기능"));
}

// ── 유틸 함수 ──────────────────────────────────────────────────────────────

fn parse_power_reference_line(line: &str) -> Option<(&str, &str, &str)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let mut parts = line.splitn(3, '\t');
    let category = parts.next()?.trim();
    let item = parts.next()?.trim();
    let value = parts.next()?.trim();
    if category.is_empty() || item.is_empty() {
        return None;
    }

    Some((category, item, value))
}

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
    let Ok(boot_dt) = chrono::NaiveDateTime::parse_from_str(&boot_wmi[..14], "%Y%m%d%H%M%S") else {
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

#[cfg(test)]
mod tests {
    use super::parse_power_reference_line;

    #[test]
    fn power_reference_line_parses_tsv() {
        assert_eq!(
            parse_power_reference_line("전원 참고\tUSB 선택적 절전\tAC: 사용 / DC: 사용 안 함"),
            Some(("전원 참고", "USB 선택적 절전", "AC: 사용 / DC: 사용 안 함"))
        );
    }

    #[test]
    fn power_reference_line_preserves_tabs_after_value_split() {
        assert_eq!(
            parse_power_reference_line("전원 관리 장치\tUSB Root Hub\t허용\t추가"),
            Some(("전원 관리 장치", "USB Root Hub", "허용\t추가"))
        );
    }

    #[test]
    fn power_reference_line_ignores_incomplete_rows() {
        assert_eq!(parse_power_reference_line("전원 참고\t"), None);
        assert_eq!(parse_power_reference_line(""), None);
    }
}
