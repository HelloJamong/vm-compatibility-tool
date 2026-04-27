/// CSV 내보내기 커맨드
use crate::models::{
    installed_program::InstalledProgramItem, system_info::SystemInfoItem,
    virtualization::VirtualizationItem,
};
use crate::services::log_service;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct InspectionExportOutput {
    pub system_csv_path: String,
    pub virtualization_csv_path: String,
    pub installed_programs_csv_path: String,
}

#[tauri::command]
pub async fn export_csv(
    file_path: String,
    data_type: String,
    system_items: Option<Vec<SystemInfoItem>>,
    virt_items: Option<Vec<VirtualizationItem>>,
    installed_program_items: Option<Vec<InstalledProgramItem>>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        write_csv(
            &file_path,
            &data_type,
            system_items,
            virt_items,
            installed_program_items,
        )
    })
    .await
    .map_err(|e| format!("작업 오류: {e}"))?
    .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn export_csv_auto(
    data_type: String,
    system_items: Option<Vec<SystemInfoItem>>,
    virt_items: Option<Vec<VirtualizationItem>>,
    installed_program_items: Option<Vec<InstalledProgramItem>>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = build_auto_csv_path(&data_type)?;
        write_csv(
            path.to_string_lossy().as_ref(),
            &data_type,
            system_items,
            virt_items,
            installed_program_items,
        )?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|e| format!("작업 오류: {e}"))?
    .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn export_inspection_csvs_auto(
    system_items: Vec<SystemInfoItem>,
    virt_items: Vec<VirtualizationItem>,
    installed_program_items: Vec<InstalledProgramItem>,
) -> Result<InspectionExportOutput, String> {
    tokio::task::spawn_blocking(move || {
        let system_path = build_auto_csv_path("system")?;
        let virt_path = build_auto_csv_path("virtualization")?;
        let installed_programs_path = build_auto_csv_path("installed_programs")?;

        write_csv(
            system_path.to_string_lossy().as_ref(),
            "system",
            Some(system_items),
            None,
            None,
        )?;
        write_csv(
            virt_path.to_string_lossy().as_ref(),
            "virtualization",
            None,
            Some(virt_items),
            None,
        )?;
        write_csv(
            installed_programs_path.to_string_lossy().as_ref(),
            "installed_programs",
            None,
            None,
            Some(installed_program_items),
        )?;

        Ok(InspectionExportOutput {
            system_csv_path: system_path.to_string_lossy().into_owned(),
            virtualization_csv_path: virt_path.to_string_lossy().into_owned(),
            installed_programs_csv_path: installed_programs_path.to_string_lossy().into_owned(),
        })
    })
    .await
    .map_err(|e| format!("작업 오류: {e}"))?
    .map_err(|e: anyhow::Error| e.to_string())
}

fn build_auto_csv_path(data_type: &str) -> anyhow::Result<PathBuf> {
    let dir = log_service::operation_log_dir()
        .ok_or_else(|| anyhow::anyhow!("저장 경로를 찾을 수 없습니다"))?;
    fs::create_dir_all(&dir)?;

    let ts = chrono::Local::now().format("%y%m%d_%H%M%S");
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN".to_string());
    let filename = match data_type {
        "system" => format!("{ts}_{hostname}-SystemInfo.csv"),
        "virtualization" => format!("{ts}_{hostname}-reg.csv"),
        "installed_programs" => format!("{ts}_{hostname}-Programs.csv"),
        other => format!("{ts}_{hostname}-{other}.csv"),
    };

    Ok(dir.join(filename))
}

fn write_csv(
    path: &str,
    data_type: &str,
    system_items: Option<Vec<SystemInfoItem>>,
    virt_items: Option<Vec<VirtualizationItem>>,
    installed_program_items: Option<Vec<InstalledProgramItem>>,
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
        "installed_programs" => {
            content.push_str("설치된 프로그램 이름,제조사,날짜\n");
            if let Some(items) = installed_program_items {
                for item in items {
                    content.push_str(&format!(
                        "{},{},{}\n",
                        escape_csv(&item.name),
                        escape_csv(&item.publisher),
                        escape_csv(&item.install_date),
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
