use serde::{Deserialize, Serialize};

/// 설치된 프로그램/앱 CSV 행
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledProgramItem {
    pub name: String,
    pub publisher: String,
    pub install_date: String,
}

impl InstalledProgramItem {
    pub fn new(name: &str, publisher: &str, install_date: &str) -> Self {
        Self {
            name: name.to_string(),
            publisher: publisher.to_string(),
            install_date: install_date.to_string(),
        }
    }
}
