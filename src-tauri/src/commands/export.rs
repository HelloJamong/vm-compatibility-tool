/// CSV 내보내기 커맨드

use crate::models::{system_info::SystemInfoItem, virtualization::VirtualizationItem};
use std::fs;

#[tauri::command]
pub async fn export_csv(
    file_path: String,
    data_type: String,
    system_items: Option<Vec<SystemInfoItem>>,
    virt_items: Option<Vec<VirtualizationItem>>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        write_csv(&file_path, &data_type, system_items, virt_items)
    })
    .await
    .map_err(|e| format!("작업 오류: {e}"))?
    .map_err(|e: anyhow::Error| e.to_string())
}

fn write_csv(
    path: &str,
    data_type: &str,
    system_items: Option<Vec<SystemInfoItem>>,
    virt_items: Option<Vec<VirtualizationItem>>,
) -> anyhow::Result<()> {
    let mut content = String::new();

    // UTF-8 BOM (한글 깨짐 방지)
    content.push('\u{FEFF}');

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    content.push_str(&format!("# VM Compatibility Tool\n"));
    content.push_str(&format!("# 생성일시: {now}\n\n"));

    match data_type {
        "system" => {
            content.push_str("항목,세부 정보,값\n");
            if let Some(items) = system_items {
                for item in items {
                    content.push_str(&format!(
                        "{},{},{}\n",
                        escape_csv(&item.category),
                        escape_csv(&item.item),
                        escape_csv(&item.value),
                    ));
                }
            }
        }
        "virtualization" => {
            content.push_str("항목,상태,상세 정보,권장사항\n");
            if let Some(items) = virt_items {
                for item in items {
                    content.push_str(&format!(
                        "{},{},{},{}\n",
                        escape_csv(&item.category),
                        escape_csv(&item.status),
                        escape_csv(&item.details),
                        escape_csv(&item.recommendation),
                    ));
                }
            }
        }
        _ => {}
    }

    // UTF-8 BOM 포함 파일 쓰기
    fs::write(path, content.as_bytes())?;
    Ok(())
}

fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('\n') || field.contains('"') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
