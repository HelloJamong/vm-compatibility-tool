/// WMI 서비스 — Windows Management Instrumentation 쿼리 래퍼
///
/// Phase 0 PoC: Win32_Processor, Win32_ComputerSystem, Win32_OperatingSystem
///
/// wmi crate는 COM 초기화가 필요하므로 반드시 tokio::task::spawn_blocking 내에서 호출
#[cfg(windows)]
pub mod windows {
    use anyhow::{Context, Result};
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    // ── WMI 구조체 정의 ────────────────────────────────────────────────────

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_Processor")]
    pub struct Win32Processor {
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "NumberOfCores")]
        pub number_of_cores: u32,
        #[serde(rename = "NumberOfLogicalProcessors")]
        pub number_of_logical_processors: u32,
        #[serde(rename = "MaxClockSpeed")]
        pub max_clock_speed: u32,
        #[serde(rename = "Manufacturer")]
        pub manufacturer: String,
        #[serde(rename = "VirtualizationFirmwareEnabled")]
        pub virtualization_firmware_enabled: bool,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_ComputerSystem")]
    pub struct Win32ComputerSystem {
        #[serde(rename = "TotalPhysicalMemory")]
        pub total_physical_memory: u64,
        #[serde(rename = "Manufacturer")]
        pub manufacturer: String,
        #[serde(rename = "Model")]
        pub model: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_OperatingSystem")]
    pub struct Win32OperatingSystem {
        #[serde(rename = "LastBootUpTime")]
        pub last_boot_up_time: String,
        #[serde(rename = "FreePhysicalMemory")]
        pub free_physical_memory: u64,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_VideoController")]
    pub struct Win32VideoController {
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "AdapterRAM")]
        pub adapter_ram: Option<u64>,
        #[serde(rename = "DriverVersion")]
        pub driver_version: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_BaseBoard")]
    pub struct Win32BaseBoard {
        #[serde(rename = "Manufacturer")]
        pub manufacturer: String,
        #[serde(rename = "Product")]
        pub product: String,
        #[serde(rename = "SerialNumber")]
        pub serial_number: Option<String>,
    }

    // ── MSFT_PhysicalDisk (SSD/HDD 판별) ──────────────────────────────────
    #[derive(Deserialize, Debug)]
    #[serde(rename = "MSFT_PhysicalDisk")]
    pub struct MsftPhysicalDisk {
        #[serde(rename = "FriendlyName")]
        pub friendly_name: String,
        #[serde(rename = "MediaType")]
        pub media_type: Option<u16>,
        #[serde(rename = "BusType")]
        pub bus_type: Option<u16>,
    }

    // ── WMI 연결 헬퍼 ─────────────────────────────────────────────────────

    /// 기본 WMI 연결 (root\cimv2)
    pub fn connect() -> Result<(COMLibrary, WMIConnection)> {
        let com = COMLibrary::new().context("COM 초기화 실패")?;
        let wmi = WMIConnection::new(com.into()).context("WMI 연결 실패")?;
        Ok((com, wmi))  // NOTE: com은 wmi와 함께 drop되어야 함 — 같이 반환
    }

    /// Storage 네임스페이스 WMI 연결 (MSFT_PhysicalDisk용)
    pub fn connect_storage() -> Result<(COMLibrary, WMIConnection)> {
        let com = COMLibrary::new().context("COM 초기화 실패")?;
        let wmi = WMIConnection::with_namespace_path(
            "root\\Microsoft\\Windows\\Storage",
            com.into(),
        )
        .context("Storage WMI 연결 실패")?;
        Ok((com, wmi))
    }

    // ── Phase 0 PoC 검증 함수 ─────────────────────────────────────────────

    /// [PoC] CPU 정보 조회 — WMI Win32_Processor
    pub fn poc_get_cpu_info() -> Result<Vec<Win32Processor>> {
        let (_com, wmi) = connect()?;
        let processors: Vec<Win32Processor> = wmi.query().context("Win32_Processor 쿼리 실패")?;
        Ok(processors)
    }

    /// [PoC] 메모리/시스템 정보 조회 — WMI Win32_ComputerSystem
    pub fn poc_get_system_info() -> Result<Vec<Win32ComputerSystem>> {
        let (_com, wmi) = connect()?;
        let systems: Vec<Win32ComputerSystem> =
            wmi.query().context("Win32_ComputerSystem 쿼리 실패")?;
        Ok(systems)
    }
}

// Windows가 아닌 환경에서는 스텁 제공 (cargo check 통과용)
#[cfg(not(windows))]
pub mod windows {
    use anyhow::Result;

    pub fn poc_get_cpu_info() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn poc_get_system_info() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }
}
