/// 프로세스 서비스 — dism.exe / bcdedit.exe / shutdown.exe 실행 래퍼
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// PowerShell 단일 호출 최대 대기 시간.
/// WMI/WUA/이벤트 로그 관련 스크립트가 일부 PC에서 무한 대기하는 사례가 있어
/// 초과 시 프로세스를 강제 종료하고 해당 항목만 건너뛴다.
const POWERSHELL_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl ProcessResult {
    fn from_output(output: std::process::Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    fn error(message: &str) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: message.to_string(),
            exit_code: -1,
        }
    }
}

/// DISM으로 Windows 기능 상태 조회
pub fn get_feature_state(feature_name: &str) -> ProcessResult {
    let args = feature_state_args(feature_name);
    Command::new("dism.exe")
        .args(args)
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

/// DISM으로 Windows 기능 비활성화
pub fn disable_feature(feature_name: &str) -> ProcessResult {
    let args = disable_feature_args(feature_name);
    Command::new("dism.exe")
        .args(args)
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

fn feature_state_args(feature_name: &str) -> Vec<String> {
    vec![
        "/English".to_string(),
        "/online".to_string(),
        "/get-featureinfo".to_string(),
        format!("/featurename:{feature_name}"),
    ]
}

fn disable_feature_args(feature_name: &str) -> Vec<String> {
    vec![
        "/English".to_string(),
        "/online".to_string(),
        "/disable-feature".to_string(),
        format!("/featurename:{feature_name}"),
        "/norestart".to_string(),
    ]
}

pub fn parse_dism_feature_state(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case("State")
            .then(|| value.trim().to_string())
    })
}

/// bcdedit으로 hypervisorlaunchtype 비활성화
pub fn disable_hypervisor_launch() -> ProcessResult {
    Command::new("bcdedit.exe")
        .args(["/set", "hypervisorlaunchtype", "off"])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

/// bcdedit으로 현재 hypervisorlaunchtype 상태 확인
pub fn get_hypervisor_launch_type() -> String {
    let result = Command::new("bcdedit.exe")
        .args(["/enum", "{current}"])
        .creation_flags_no_window()
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                return format_process_output_error(&output);
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // BCD에 항목이 없으면 기본값 Auto — "확인 불가"(읽기 실패)와 구분해 "미설정" 반환
            parse_bcdedit_value(&stdout, "hypervisorlaunchtype")
                .unwrap_or_else(|| "미설정".to_string())
        }
        Err(e) => format!("오류: {}", e),
    }
}

/// bcdedit으로 vsmlaunchtype 비활성화
pub fn disable_vsm_launch() -> ProcessResult {
    Command::new("bcdedit.exe")
        .args(["/set", "vsmlaunchtype", "off"])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

/// bcdedit으로 현재 vsmlaunchtype 상태 확인 — BCD에 없으면 "미설정" 반환
pub fn get_vsm_launch_type() -> String {
    let result = Command::new("bcdedit.exe")
        .args(["/enum", "{current}"])
        .creation_flags_no_window()
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                return format_process_output_error(&output);
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            parse_bcdedit_value(&stdout, "vsmlaunchtype").unwrap_or_else(|| "미설정".to_string())
        }
        Err(e) => format!("오류: {}", e),
    }
}

fn format_process_output_error(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = if !stderr.trim().is_empty() {
        stderr.trim()
    } else if !stdout.trim().is_empty() {
        stdout.trim()
    } else {
        "출력 없음"
    };
    format!("오류: {message}")
}

/// bcdedit /enum 출력에서 특정 키의 값을 파싱합니다.
/// 키·값 사이에 여러 공백/탭이 있을 수 있으므로 split_whitespace로 처리합니다.
fn parse_bcdedit_value(output: &str, key: &str) -> Option<String> {
    let key_lower = key.to_lowercase();
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains(&key_lower) {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 2 {
                return Some(tokens[1..].join(" "));
            }
        }
    }
    None
}

/// PowerShell 스크립트 실행 (타임아웃 포함)
pub fn run_powershell(script: &str) -> ProcessResult {
    run_powershell_with_timeout(script, POWERSHELL_TIMEOUT)
}

fn run_powershell_with_timeout(script: &str, timeout: Duration) -> ProcessResult {
    let wrapped_script = wrap_powershell_script(script);

    let spawn = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped_script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags_no_window()
        .spawn();

    let mut child = match spawn {
        Ok(child) => child,
        Err(e) => return ProcessResult::error(&e.to_string()),
    };

    // 파이프 버퍼가 가득 차서 자식이 멈추는 것을 막기 위해 별도 스레드로 배출한다.
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let out_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stdout_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });
    let err_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stderr_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break None,
        }
    };

    let stdout = String::from_utf8_lossy(&out_handle.join().unwrap_or_default()).to_string();
    let stderr = String::from_utf8_lossy(&err_handle.join().unwrap_or_default()).to_string();

    if timed_out {
        return ProcessResult {
            success: false,
            stdout,
            stderr: format!(
                "PowerShell 실행 시간 초과({}초) — 해당 항목을 건너뜁니다",
                timeout.as_secs()
            ),
            exit_code: -1,
        };
    }

    match status {
        Some(status) => ProcessResult {
            success: status.success(),
            stdout,
            stderr,
            exit_code: status.code().unwrap_or(-1),
        },
        None => ProcessResult::error("PowerShell 프로세스 상태 확인 실패"),
    }
}

fn wrap_powershell_script(script: &str) -> String {
    format!(
        r#"
[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [Console]::OutputEncoding

{}
"#,
        script
    )
}

/// 시스템 재부팅 (5초 후)
pub fn schedule_reboot() -> ProcessResult {
    Command::new("shutdown.exe")
        .args(["/r", "/t", "5", "/c", "VM Compatibility Tool에 의한 재부팅"])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

/// 예약된 재부팅 취소
pub fn cancel_reboot() -> ProcessResult {
    Command::new("shutdown.exe")
        .args(["/a"])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

// Windows에서 콘솔 창 숨기기 위한 트레이트 확장
trait CommandExt {
    fn creation_flags_no_window(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn creation_flags_no_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW = 0x08000000
            self.creation_flags(0x08000000);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        disable_feature_args, feature_state_args, parse_bcdedit_value, parse_dism_feature_state,
        wrap_powershell_script,
    };

    #[cfg(windows)]
    #[test]
    fn powershell_call_that_hangs_is_killed_after_timeout() {
        use super::run_powershell_with_timeout;
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let result = run_powershell_with_timeout("Start-Sleep -Seconds 30", Duration::from_secs(2));

        assert!(!result.success);
        assert!(result.stderr.contains("시간 초과"));
        assert!(start.elapsed() < Duration::from_secs(10), "프로세스가 강제 종료되지 않음");
    }

    #[test]
    fn dism_feature_commands_force_english_output() {
        assert_eq!(feature_state_args("Example").first().unwrap(), "/English");
        assert_eq!(disable_feature_args("Example").first().unwrap(), "/English");
    }

    #[test]
    fn dism_feature_state_parser_handles_whitespace() {
        assert_eq!(
            parse_dism_feature_state("Feature Name : Example\r\nState : Enabled\r\n"),
            Some("Enabled".to_string())
        );
    }

    #[test]
    fn powershell_wrapper_forces_utf8_io() {
        let wrapped = wrap_powershell_script("Write-Output 'ok'");

        assert!(wrapped.contains("[Console]::InputEncoding"));
        assert!(wrapped.contains("[Console]::OutputEncoding"));
        assert!(wrapped.contains("$OutputEncoding"));
        assert!(wrapped.contains("Write-Output 'ok'"));
    }

    #[test]
    fn bcdedit_parses_multi_space_separated_value() {
        let output = "Windows Boot Loader\n-------------------\nhypervisorlaunchtype    Auto\n";
        assert_eq!(
            parse_bcdedit_value(output, "hypervisorlaunchtype"),
            Some("Auto".to_string())
        );
    }

    #[test]
    fn bcdedit_parses_tab_separated_value() {
        let output = "hypervisorlaunchtype\tOff\n";
        assert_eq!(
            parse_bcdedit_value(output, "hypervisorlaunchtype"),
            Some("Off".to_string())
        );
    }

    #[test]
    fn bcdedit_returns_none_when_key_absent() {
        let output = "identifier    {current}\ndevice    partition=C:\n";
        assert_eq!(parse_bcdedit_value(output, "vsmlaunchtype"), None);
    }

    #[test]
    fn bcdedit_parses_vsm_launch_type() {
        let output = "vsmlaunchtype    Auto\n";
        assert_eq!(
            parse_bcdedit_value(output, "vsmlaunchtype"),
            Some("Auto".to_string())
        );
    }
}
