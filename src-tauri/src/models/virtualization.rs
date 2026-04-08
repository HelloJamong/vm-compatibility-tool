use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DisableGroup {
    Hyperv,
    Wsl,
    Vbs,
    CoreIsolation,
}

impl DisableGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hyperv => "hyperv",
            Self::Wsl => "wsl",
            Self::Vbs => "vbs",
            Self::CoreIsolation => "core_isolation",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VirtualizationSource {
    Unknown,
    Wmi,
    Feature,
    Bcd,
    Registry,
}

/// 가상화 점검 테이블 행
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationItem {
    pub category: String,
    pub status: String,
    pub details: String,
    pub recommendation: String,
    pub disable_group: Option<DisableGroup>,
    pub source_type: VirtualizationSource,
    pub action_required: bool,
    pub manifest_id: Option<String>,
}

impl VirtualizationItem {
    pub fn new(category: &str, status: &str, details: &str, recommendation: &str) -> Self {
        Self {
            category: category.to_string(),
            status: status.to_string(),
            details: details.to_string(),
            recommendation: recommendation.to_string(),
            disable_group: None,
            source_type: VirtualizationSource::Unknown,
            action_required: false,
            manifest_id: None,
        }
    }

    pub fn with_disable_group(
        mut self,
        disable_group: DisableGroup,
        action_required: bool,
    ) -> Self {
        self.disable_group = Some(disable_group);
        self.action_required = action_required;
        self
    }

    pub fn with_source(mut self, source_type: VirtualizationSource) -> Self {
        self.source_type = source_type;
        self
    }

    pub fn with_manifest_id(mut self, manifest_id: &str) -> Self {
        self.manifest_id = Some(manifest_id.to_string());
        self
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

/// 비활성화 옵션 — selective 모드에서 필요한 항목만 실행
///
/// 프론트엔드가 가상화 점검 결과를 기반으로 계산하여 전달.
/// None이면 모든 항목 실행 (전체 모드).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableOptions {
    pub hyperv: bool,
    pub wsl: bool,
    pub vbs: bool,
    pub core_isolation: bool,
}

impl DisableOptions {
    pub fn all() -> Self {
        Self {
            hyperv: true,
            wsl: true,
            vbs: true,
            core_isolation: true,
        }
    }

    /// 하나라도 실행 대상이 있는지 확인
    pub fn any(&self) -> bool {
        self.hyperv || self.wsl || self.vbs || self.core_isolation
    }
}
