use serde::{Deserialize, Serialize};

/// 시스템 정보 테이블 행 (프론트엔드 DataGrid 바인딩용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoItem {
    pub category: String,
    pub item: String,
    pub value: String,
}

impl SystemInfoItem {
    pub fn new(category: &str, item: &str, value: &str) -> Self {
        Self {
            category: category.to_string(),
            item: item.to_string(),
            value: value.to_string(),
        }
    }

    pub fn error(category: &str, message: &str) -> Self {
        Self {
            category: category.to_string(),
            item: "오류".to_string(),
            value: message.to_string(),
        }
    }
}
