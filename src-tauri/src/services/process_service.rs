/// 프로세스 서비스 — dism.exe / bcdedit.exe / shutdown.exe 실행 래퍼
use std::process::Command;

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
    Command::new("dism.exe")
        .args([
            "/online",
            "/get-featureinfo",
            &format!("/featurename:{}", feature_name),
        ])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
}

/// DISM으로 Windows 기능 비활성화
pub fn disable_feature(feature_name: &str) -> ProcessResult {
    Command::new("dism.exe")
        .args([
            "/online",
            "/disable-feature",
            &format!("/featurename:{}", feature_name),
            "/norestart",
        ])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
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
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // "hypervisorlaunchtype     Auto" 또는 "Off" 파싱
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("hypervisorlaunchtype") {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        return parts[1].trim().to_string();
                    }
                }
            }
            "확인 불가".to_string()
        }
        Err(e) => format!("오류: {}", e),
    }
}

/// PowerShell 스크립트 실행
pub fn run_powershell(script: &str) -> ProcessResult {
    let wrapped_script = wrap_powershell_script(script);

    Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped_script,
        ])
        .creation_flags_no_window()
        .output()
        .map(ProcessResult::from_output)
        .unwrap_or_else(|e| ProcessResult::error(&e.to_string()))
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
    use super::wrap_powershell_script;

    #[test]
    fn powershell_wrapper_forces_utf8_io() {
        let wrapped = wrap_powershell_script("Write-Output 'ok'");

        assert!(wrapped.contains("[Console]::InputEncoding"));
        assert!(wrapped.contains("[Console]::OutputEncoding"));
        assert!(wrapped.contains("$OutputEncoding"));
        assert!(wrapped.contains("Write-Output 'ok'"));
    }
}
