/// 디스크 타입 감지 서비스 — SSD/HDD 판별
///
/// C# 원본 GetDriveMediaType() L1014 의 Rust 구현
/// 우선순위: MSFT_PhysicalDisk → Win32_DiskDrive → Registry → 알 수 없음

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum DiskType {
    SsdNvme,
    Ssd,
    Hdd,
    Unknown,
}

impl std::fmt::Display for DiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskType::SsdNvme => write!(f, "SSD (NVMe)"),
            DiskType::Ssd => write!(f, "SSD"),
            DiskType::Hdd => write!(f, "HDD"),
            DiskType::Unknown => write!(f, "알 수 없음"),
        }
    }
}

/// MediaType 값 → DiskType 변환 (MSFT_PhysicalDisk 기준)
/// 3 = HDD, 4 = SSD, 5 = SCM
pub fn media_type_to_disk_type(media_type: u16, bus_type: Option<u16>) -> DiskType {
    match media_type {
        4 => {
            // BusType 17 = NVMe
            if bus_type == Some(17) {
                DiskType::SsdNvme
            } else {
                DiskType::Ssd
            }
        }
        3 => DiskType::Hdd,
        _ => DiskType::Unknown,
    }
}

/// 모델명 키워드로 SSD 판별 (MSFT_PhysicalDisk MediaType 없을 때 폴백)
pub fn detect_from_model_name(model: &str) -> DiskType {
    let model_upper = model.to_uppercase();

    let nvme_keywords = ["NVME", "PCIE", "980", "970", "960", "SN"];
    let ssd_keywords = [
        "SSD", "SOLID STATE", "M.2", "MSATA", "FLASH", "NAND",
        "EVO", "PRO", "MX", "BX", "SHGS",
    ];
    let hdd_keywords = ["HDD", "HARDDISK", "7200", "5400", "RPM", "WD BLUE", "BARRACUDA"];

    if nvme_keywords.iter().any(|k| model_upper.contains(k)) {
        return DiskType::SsdNvme;
    }
    if ssd_keywords.iter().any(|k| model_upper.contains(k)) {
        return DiskType::Ssd;
    }
    if hdd_keywords.iter().any(|k| model_upper.contains(k)) {
        return DiskType::Hdd;
    }

    DiskType::Unknown
}
