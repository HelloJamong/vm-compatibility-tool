use serde::{Deserialize, Serialize};

/// 가상화 점검 테이블 행
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationItem {
    pub category: String,
    pub status: String,
    pub details: String,
    pub recommendation: String,
}

impl VirtualizationItem {
    pub fn new(category: &str, status: &str, details: &str, recommendation: &str) -> Self {
        Self {
            category: category.to_string(),
            status: status.to_string(),
            details: details.to_string(),
            recommendation: recommendation.to_string(),
        }
    }
}

/// 비활성화 작업 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableResult {
    pub task: String,
    pub success: bool,
    pub message: String,
}

/// 실시간 진행 이벤트 (Tauri emit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub step: u32,
    pub total: u32,
    pub message: String,
    pub success: bool,
}
