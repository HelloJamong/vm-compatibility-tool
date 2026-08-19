/// 에러 로그 — %TEMP%\VMCompatibilityTool\error_YYYYMMDD.log
/// 운영 로그 — {exe_dir}\vmc_logs\YYYYMMDD_HHMMSS_{computer_name}.log
use crate::services::csv_service::escape_field;
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
    let dir = exe.parent()?.join("vmc_logs");
    Some(dir)
}

pub fn backup_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.join("vmc_backup");
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

/// 비활성화 조치 전/후 비교 CSV 행
pub struct DisableChangeEntry {
    pub group: String,
    pub item: String,
    pub target: String,
    pub before: String,
    pub after: String,
    pub result: String,
    pub message: String,
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

    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
    }
}

/// 레지스트리 변경 전에 복구 파일을 먼저 저장합니다.
pub fn save_registry_backup(
    backup_entries: &[RegistryBackupEntry],
) -> anyhow::Result<Option<String>> {
    if backup_entries.is_empty() {
        return Ok(None);
    }

    let bak_dir = backup_dir().ok_or_else(|| anyhow::anyhow!("백업 경로를 찾을 수 없습니다"))?;
    fs::create_dir_all(&bak_dir)?;
    let now = chrono::Local::now();
    let ts = now.format("%y%m%d_%H%M%S").to_string();
    let reg_path = bak_dir.join(format!("{}_backup.reg", ts));
    write_reg_backup(&reg_path, backup_entries, &now)?;

    Ok(Some(reg_path.to_string_lossy().into_owned()))
}

/// 운영 로그 저장
pub fn save_operation_log(log_lines: &[String]) -> anyhow::Result<String> {
    let log_dir =
        operation_log_dir().ok_or_else(|| anyhow::anyhow!("로그 경로를 찾을 수 없습니다"))?;
    fs::create_dir_all(&log_dir)?;

    let now = chrono::Local::now();
    let ts = now.format("%y%m%d_%H%M%S").to_string();
    let log_path = log_dir.join(format!("{}_{}.log", ts, computer_name()));

    write_operation_log(&log_path, log_lines, &now)?;

    Ok(log_path.to_string_lossy().into_owned())
}

/// 비활성화 조치 전/후 비교 CSV 저장
pub fn save_disable_change_csv(entries: &[DisableChangeEntry]) -> anyhow::Result<String> {
    let log_dir =
        operation_log_dir().ok_or_else(|| anyhow::anyhow!("로그 경로를 찾을 수 없습니다"))?;
    fs::create_dir_all(&log_dir)?;

    let now = chrono::Local::now();
    let ts = now.format("%y%m%d_%H%M%S").to_string();
    let cn = computer_name();
    let csv_path = log_dir.join(format!("{}_{}-DisableResult.csv", ts, cn));

    write_disable_change_csv(&csv_path, entries, &now)?;
    Ok(csv_path.to_string_lossy().into_owned())
}

fn write_disable_change_csv(
    path: &PathBuf,
    entries: &[DisableChangeEntry],
    now: &chrono::DateTime<chrono::Local>,
) -> std::io::Result<()> {
    let mut content = String::new();
    content.push('\u{FEFF}');
    content.push_str("VM Compatibility Tool — 조치 전후 비교\n");
    content.push_str(&format!("생성,{}\n", now.format("%Y-%m-%d %H:%M:%S")));
    content.push_str(&format!("컴퓨터,{}\n\n", computer_name()));
    content.push_str("작업 그룹,항목,대상,조치 전,조치 후,결과,메시지\n");

    for entry in entries {
        content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            escape_field(&entry.group),
            escape_field(&entry.item),
            escape_field(&entry.target),
            escape_field(&entry.before),
            escape_field(&entry.after),
            escape_field(&entry.result),
            escape_field(&entry.message),
        ));
    }

    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())
}

fn write_operation_log(
    path: &PathBuf,
    lines: &[String],
    now: &chrono::DateTime<chrono::Local>,
) -> std::io::Result<()> {
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

    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())
}

fn write_reg_backup(
    path: &PathBuf,
    entries: &[RegistryBackupEntry],
    now: &chrono::DateTime<chrono::Local>,
) -> std::io::Result<()> {
    let mut lines: Vec<String> = vec![
        "Windows Registry Editor Version 5.00".to_string(),
        String::new(),
        format!("; VM Compatibility Tool — 레지스트리 백업"),
        format!("; 생성: {}", now.format("%Y-%m-%d %H:%M:%S")),
        format!("; 컴퓨터: {}", computer_name()),
        format!("; 이 파일을 실행하면 변경 전 값으로 복원됩니다."),
        format!("; 주의: 조치 전 원본 값이 없던 항목(주석 처리된 항목)은 복원되지 않습니다."),
        format!(";       해당 항목을 완전히 제거하려면 레지스트리 편집기에서 직접 삭제하세요."),
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
    let mut file = fs::File::create(path)?;
    file.write_all(&encoded)
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
