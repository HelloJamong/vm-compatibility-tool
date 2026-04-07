/// 에러 로그 서비스 — %TEMP%\VMCompatibilityTool\error_YYYYMMDD.log

use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn log_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("VMCompatibilityTool");
    dir
}

/// 로그 디렉토리 생성 (없으면)
pub fn init() {
    let _ = fs::create_dir_all(log_dir());
}

/// 에러를 날짜별 로그 파일에 기록
pub fn log_error(context: &str, error: &str) {
    let dir = log_dir();
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

/// 오늘 로그 파일 경로 반환 (없으면 None)
pub fn today_log_path() -> Option<String> {
    let now = chrono::Local::now();
    let filename = format!("error_{}.log", now.format("%Y%m%d"));
    let path = log_dir().join(filename);
    if path.exists() {
        path.to_str().map(|s| s.to_string())
    } else {
        None
    }
}
