/// WMI 서비스 — Windows Management Instrumentation 쿼리 래퍼
///
/// wmi crate는 COM 초기화가 필요하므로 반드시 tokio::task::spawn_blocking 내에서 호출

#[cfg(windows)]
pub mod windows {
    use anyhow::{Context, Result};
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    // ── WMI 구조체 ─────────────────────────────────────────────────────────

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

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_DiskDrive")]
    pub struct Win32DiskDrive {
        #[serde(rename = "Model")]
        pub model: String,
        #[serde(rename = "Size")]
        pub size: Option<u64>,
        #[serde(rename = "InterfaceType")]
        pub interface_type: Option<String>,
    }

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

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_PowerPlan")]
    pub struct Win32PowerPlan {
        #[serde(rename = "ElementName")]
        pub element_name: String,
        #[serde(rename = "IsActive")]
        pub is_active: Option<bool>,
    }

    // ── WMI 연결 헬퍼 ─────────────────────────────────────────────────────

    /// 기본 WMI 연결 (root\cimv2)
    pub fn connect() -> Result<(COMLibrary, WMIConnection)> {
        let com = COMLibrary::new().context("COM 초기화 실패")?;
        let wmi = WMIConnection::new(com.into()).context("WMI 연결 실패")?;
        Ok((com, wmi))
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

    /// Power 네임스페이스 WMI 연결 (Win32_PowerPlan용)
    pub fn connect_power() -> Result<(COMLibrary, WMIConnection)> {
        let com = COMLibrary::new().context("COM 초기화 실패")?;
        let wmi = WMIConnection::with_namespace_path(
            "root\\cimv2\\power",
            com.into(),
        )
        .context("Power WMI 연결 실패")?;
        Ok((com, wmi))
    }

    // ── 쿼리 함수 ─────────────────────────────────────────────────────────

    pub fn poc_get_cpu_info() -> Result<Vec<Win32Processor>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_Processor 쿼리 실패")
    }

    pub fn poc_get_system_info() -> Result<Vec<Win32ComputerSystem>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_ComputerSystem 쿼리 실패")
    }

    pub fn get_os_info() -> Result<Vec<Win32OperatingSystem>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_OperatingSystem 쿼리 실패")
    }

    pub fn get_video_controllers() -> Result<Vec<Win32VideoController>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_VideoController 쿼리 실패")
    }

    pub fn get_baseboard_info() -> Result<Vec<Win32BaseBoard>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_BaseBoard 쿼리 실패")
    }

    pub fn get_disk_drives() -> Result<Vec<Win32DiskDrive>> {
        let (_com, wmi) = connect()?;
        wmi.query().context("Win32_DiskDrive 쿼리 실패")
    }

    pub fn get_msft_physical_disks() -> Result<Vec<MsftPhysicalDisk>> {
        let (_com, wmi) = connect_storage()?;
        wmi.query().context("MSFT_PhysicalDisk 쿼리 실패")
    }

    pub fn get_power_plans() -> Result<Vec<Win32PowerPlan>> {
        let (_com, wmi) = connect_power()?;
        wmi.query().context("Win32_PowerPlan 쿼리 실패")
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

    pub fn get_os_info() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn get_video_controllers() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn get_baseboard_info() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn get_disk_drives() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn get_msft_physical_disks() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    pub fn get_power_plans() -> Result<Vec<()>> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }
}
