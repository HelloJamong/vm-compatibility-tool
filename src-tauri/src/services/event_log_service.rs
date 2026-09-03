/// 이벤트 로그 서비스 — PowerShell을 통해 Windows 이벤트 로그 요약 수집
///
/// 수집 범위: 최근 7일간 System / Application 로그
/// Level 1=위험, 2=오류, 3=경고 건수 + 최근 오류/위험 이벤트 5건
use crate::models::system_info::SystemInfoItem;
use crate::services::process_service;

/// 이벤트 로그 요약 수집 — items에 직접 추가
pub fn collect_event_log_info(items: &mut Vec<SystemInfoItem>) {
    // 각 로그별 Level별 건수 + 최근 오류/위험 이벤트 수집
    // 출력 형식:
    //   COUNT|System|2|5|12          (로그명|위험|오류|경고)
    //   RECENT|MM-dd HH:mm|System|1001|메시지 앞 80자
    // Get-WinEvent는 -FilterHashtable로 서버측 필터링해야 빠르고, 큰/손상된 로그에서
    // 전체 열거로 멈추는 것을 피할 수 있다. -MaxEvents로 상한도 건다.
    let ps_script = r#"
$days = 7
$cutoff = (Get-Date).AddDays(-$days)

foreach ($logName in @('System', 'Application')) {
    try {
        $filter = @{ LogName = $logName; StartTime = $cutoff; Level = 1,2,3 }
        $evts = @(Get-WinEvent -FilterHashtable $filter -MaxEvents 2000 -ErrorAction Stop)
        $crit = @($evts | Where-Object { $_.Level -eq 1 }).Count
        $err  = @($evts | Where-Object { $_.Level -eq 2 }).Count
        $warn = @($evts | Where-Object { $_.Level -eq 3 }).Count
        Write-Output "COUNT|$logName|$crit|$err|$warn"
    } catch {
        if ($_.Exception.Message -match 'No events were found|이벤트를 찾을 수 없습니다') {
            Write-Output "COUNT|$logName|0|0|0"
        } else {
            Write-Output "COUNT|$logName|오류|0|0"
        }
    }
}

try {
    $filter = @{ LogName = 'System','Application'; StartTime = $cutoff; Level = 1,2 }
    $recent = @(Get-WinEvent -FilterHashtable $filter -MaxEvents 5 -ErrorAction Stop)
    foreach ($e in $recent) {
        $firstLine = ($e.Message -split "`r?`n")[0] -replace '\|','-'
        $msg = if ($firstLine.Length -gt 80) { $firstLine.Substring(0,80) + '...' } else { $firstLine }
        $time = $e.TimeCreated.ToString('MM-dd HH:mm')
        Write-Output "RECENT|$time|$($e.LogName)|$($e.Id)|$msg"
    }
} catch {}
"#;

    let result = process_service::run_powershell(ps_script);

    if !result.success && result.stdout.is_empty() {
        items.push(SystemInfoItem::error("이벤트 로그", "PowerShell 실행 실패"));
        return;
    }

    let mut has_count = false;

    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        match parts.as_slice() {
            ["COUNT", log, crit, err, warn] => {
                let cat = format!("이벤트 로그 ({})", log);
                items.push(SystemInfoItem::new(&cat, "위험", crit));
                items.push(SystemInfoItem::new(&cat, "오류", err));
                items.push(SystemInfoItem::new(&cat, "경고", warn));
                has_count = true;
            }
            ["RECENT", time, log, id, msg] => {
                items.push(SystemInfoItem::new(
                    &format!("최근 오류/위험 ({})", time),
                    &format!("[{}] ID:{}", log, id),
                    msg,
                ));
            }
            _ => {}
        }
    }

    if !has_count {
        items.push(SystemInfoItem::error(
            "이벤트 로그",
            "수집 실패 — 관리자 권한 확인",
        ));
    }
}
