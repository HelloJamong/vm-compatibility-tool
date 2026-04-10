/// 에러 로그 — %TEMP%\VMCompatibilityTool\error_YYYYMMDD.log
/// 운영 로그 — {exe_dir}\logs\YYYYMMDD_HHMMSS_{computer_name}.log

use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn error_log_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("VMCompatibilityTool");
    dir
}

pub fn operation_log_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.join("logs");
    Some(dir)
}

fn computer_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN".to_string())
}

/// 레지스트리 백업 항목 (변경 전 원본값)
pub struct RegistryBackupEntry {
    pub path: String,
    pub value_name: String,
    /// None = 키 자체가 존재하지 않음
    pub value: Option<u32>,
}

/// 에러 로그 디렉토리 초기화
pub fn init() {
    let _ = fs::create_dir_all(error_log_dir());
}

/// 에러를 날짜별 로그 파일에 기록
pub fn log_error(context: &str, error: &str) {
    let dir = error_log_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    let now = chrono::Local::now();
    let filename = format!("error_{}.log", now.format("%Y%m%d"));
    let path = dir.join(filename);

    let entry = format!(
        "[{}] [{}]\n{}\n\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        context,
        error
    );

    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(entry.as_bytes());
    }
}

/// 운영 로그 + 레지스트리 백업 파일 저장
/// 반환: (log_path, backup_path) — 실패 시 None
pub fn save_operation_log(
    log_lines: &[String],
    backup_entries: &[RegistryBackupEntry],
) -> Option<(String, String)> {
    let dir = operation_log_dir()?;
    fs::create_dir_all(&dir).ok()?;

    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d_%H%M%S").to_string();
    let cn = computer_name();

    let log_path = dir.join(format!("{}_{}.log", ts, cn));
    let reg_path = dir.join(format!("{}_{}_backup.reg", ts, cn));

    write_operation_log(&log_path, log_lines, &now)?;
    write_reg_backup(&reg_path, backup_entries, &now)?;

    Some((
        log_path.to_string_lossy().into_owned(),
        reg_path.to_string_lossy().into_owned(),
    ))
}

fn write_operation_log(
    path: &PathBuf,
    lines: &[String],
    now: &chrono::DateTime<chrono::Local>,
) -> Option<()> {
    let header = format!(
        "VM Compatibility Tool — 운영 로그\n생성: {}\n컴퓨터: {}\n{}\n\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        computer_name(),
        "=".repeat(60)
    );

    let mut content = header;
    for line in lines {
        content.push_str(line);
        content.push('\n');
    }

    let mut file = fs::File::create(path).ok()?;
    file.write_all(content.as_bytes()).ok()?;
    Some(())
}

fn write_reg_backup(
    path: &PathBuf,
    entries: &[RegistryBackupEntry],
    now: &chrono::DateTime<chrono::Local>,
) -> Option<()> {
    let mut lines: Vec<String> = vec![
        "Windows Registry Editor Version 5.00".to_string(),
        String::new(),
        format!("; VM Compatibility Tool — 레지스트리 백업"),
        format!("; 생성: {}", now.format("%Y-%m-%d %H:%M:%S")),
        format!("; 컴퓨터: {}", computer_name()),
        format!("; 이 파일을 실행하면 변경 전 값으로 복원됩니다."),
        String::new(),
    ];

    let mut current_key = String::new();
    for entry in entries {
        let full_key = format!("HKEY_LOCAL_MACHINE\\{}", entry.path);
        if full_key != current_key {
            if !current_key.is_empty() {
                lines.push(String::new());
            }
            lines.push(format!("[{}]", full_key));
            current_key = full_key;
        }
        match entry.value {
            Some(v) => lines.push(format!("\"{}\"=dword:{:08x}", entry.value_name, v)),
            None => lines.push(format!(
                "; \"{}\" — 원본 값 없음 (복원 불필요)",
                entry.value_name
            )),
        }
    }

    let text = lines.join("\r\n");
    let encoded = encode_utf16le_with_bom(&text);
    let mut file = fs::File::create(path).ok()?;
    file.write_all(&encoded).ok()?;
    Some(())
}

fn encode_utf16le_with_bom(s: &str) -> Vec<u8> {
    // BOM: FF FE (UTF-16 LE)
    let mut bytes = vec![0xFF_u8, 0xFE_u8];
    for unit in s.encode_utf16() {
        bytes.push((unit & 0xFF) as u8);
        bytes.push((unit >> 8) as u8);
    }
    bytes
}
