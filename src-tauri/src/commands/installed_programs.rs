use crate::models::installed_program::InstalledProgramItem;
use crate::services::installed_program_service;

#[tauri::command]
pub async fn get_installed_programs() -> Result<Vec<InstalledProgramItem>, String> {
    tokio::task::spawn_blocking(installed_program_service::collect_installed_programs)
        .await
        .map_err(|e| format!("작업 실행 오류: {e}"))
}
