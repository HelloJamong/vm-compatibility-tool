/// 레지스트리 서비스 — Windows Registry 읽기/쓰기 래퍼
///
/// Phase 0 PoC: DeviceGuard VBS 상태 확인, Hyper-V 상태 확인

#[cfg(windows)]
pub mod windows {
    use anyhow::Context;
    use winreg::{enums::*, RegKey};

    /// 단일 DWORD 값 읽기 — 키 또는 값이 없으면 None
    pub fn get_dword(path: &str, name: &str) -> Option<u32> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(path)
            .and_then(|key| key.get_value::<u32, _>(name))
            .ok()
    }

    /// Windows 버전/빌드 정보 (Registry 기반)
    pub fn get_windows_version() -> WindowsVersionInfo {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            .ok();

        let get = |k: &RegKey, name: &str| -> String {
            k.get_value::<String, _>(name).unwrap_or_default()
        };
        let get_u32 = |k: &RegKey, name: &str| -> u32 { k.get_value::<u32, _>(name).unwrap_or(0) };

        match key {
            Some(k) => {
                let product_name = get(&k, "ProductName");
                let display_version = get(&k, "DisplayVersion");
                let current_build = get(&k, "CurrentBuild");
                let ubr = get_u32(&k, "UBR");

                let build_number: u32 = current_build.parse().unwrap_or(0);
                let os_name = if build_number >= 22000 {
                    "Windows 11"
                } else if build_number >= 10240 {
                    "Windows 10"
                } else {
                    "Windows"
                };

                // 24H2 빌드 번호 보정 (C# 기존 로직 유지)
                let version = if os_name == "Windows 11"
                    && build_number >= 26100
                    && display_version == "23H2"
                {
                    "24H2".to_string()
                } else {
                    display_version
                };

                // Windows 11에서도 ProductName이 "Windows 10 Pro"를 반환하는 MS 버그 대응:
                // OS 이름 접두사를 제거하고 실제 os_name과 조합
                let edition_suffix = product_name
                    .strip_prefix("Windows 10 ")
                    .or_else(|| product_name.strip_prefix("Windows 11 "))
                    .or_else(|| product_name.strip_prefix("Windows "))
                    .unwrap_or(&product_name);
                let product_display = format!("{} {}", os_name, edition_suffix);

                let install_date_raw = get_u32(&k, "InstallDate");

                // InstallLanguage 키가 없는 시스템을 위해 Nls\Language\Default로 폴백
                let install_language_raw = get(&k, "InstallLanguage");
                let install_language = if !install_language_raw.is_empty() {
                    install_language_raw
                } else {
                    RegKey::predef(HKEY_LOCAL_MACHINE)
                        .open_subkey(r"SYSTEM\CurrentControlSet\Control\Nls\Language")
                        .and_then(|lk| lk.get_value::<String, _>("Default"))
                        .unwrap_or_default()
                };

                let arch = RegKey::predef(HKEY_LOCAL_MACHINE)
                    .open_subkey(
                        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                    )
                    .and_then(|ek| ek.get_value::<String, _>("PROCESSOR_ARCHITECTURE"))
                    .unwrap_or_default();

                WindowsVersionInfo {
                    os_name: os_name.to_string(),
                    product_name: product_display,
                    display_version: version,
                    build_number: current_build,
                    ubr,
                    install_date: unix_ts_to_date(install_date_raw),
                    install_language: lcid_to_name(&install_language),
                    architecture: arch_to_display(&arch),
                }
            }
            None => WindowsVersionInfo::unknown(),
        }
    }

    /// 레지스트리 키가 존재하고 서브키를 하나 이상 가지면 true
    pub fn key_has_subkeys(path: &str) -> bool {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(path)
            .map(|key| key.enum_keys().any(|_| true))
            .unwrap_or(false)
    }

    /// 레지스트리 키의 서브키 이름 목록 반환
    pub fn list_subkeys(path: &str) -> Vec<String> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(path)
            .map(|key| key.enum_keys().filter_map(|k| k.ok()).collect())
            .unwrap_or_default()
    }

    /// 레지스트리 DWORD 값 쓰기 (비활성화 작업용)
    pub fn set_dword(path: &str, name: &str, value: u32) -> anyhow::Result<()> {
        let (key, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .create_subkey(path)
            .with_context(|| format!("레지스트리 키 생성 실패: {path}"))?;
        key.set_value(name, &value)
            .with_context(|| format!("값 설정 실패: {path}\\{name}"))?;
        Ok(())
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct WindowsVersionInfo {
        pub os_name: String,
        pub product_name: String,
        pub display_version: String,
        pub build_number: String,
        pub ubr: u32,
        pub install_date: String,
        pub install_language: String,
        pub architecture: String,
    }

    impl WindowsVersionInfo {
        fn unknown() -> Self {
            Self {
                os_name: "알 수 없음".to_string(),
                product_name: "알 수 없음".to_string(),
                display_version: "알 수 없음".to_string(),
                build_number: "0".to_string(),
                ubr: 0,
                install_date: "알 수 없음".to_string(),
                install_language: "알 수 없음".to_string(),
                architecture: "알 수 없음".to_string(),
            }
        }
    }

    fn unix_ts_to_date(ts: u32) -> String {
        if ts == 0 {
            return "알 수 없음".to_string();
        }
        chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "알 수 없음".to_string())
    }

    fn arch_to_display(arch: &str) -> String {
        match arch.to_uppercase().as_str() {
            "AMD64" => "x64 (AMD64)".to_string(),
            "ARM64" => "ARM64".to_string(),
            "X86" => "x86 (32비트)".to_string(),
            other if !other.is_empty() => other.to_string(),
            _ => "알 수 없음".to_string(),
        }
    }

    fn lcid_to_name(lcid_hex: &str) -> String {
        match lcid_hex.to_lowercase().trim_start_matches('0') {
            "409" => "영어 (미국)".to_string(),
            "412" => "한국어".to_string(),
            "411" => "일본어".to_string(),
            "804" => "중국어 (간체)".to_string(),
            "404" => "중국어 (번체)".to_string(),
            other if !other.is_empty() => lcid_hex.to_string(),
            _ => "알 수 없음".to_string(),
        }
    }
}

#[cfg(not(windows))]
pub mod windows {
    use anyhow::Result;

    pub fn get_dword(_path: &str, _name: &str) -> Option<u32> {
        None
    }
    pub fn key_has_subkeys(_path: &str) -> bool {
        false
    }
    pub fn list_subkeys(_path: &str) -> Vec<String> {
        vec![]
    }
    pub fn set_dword(_path: &str, _name: &str, _value: u32) -> Result<()> {
        Err(anyhow::anyhow!("Windows 전용 기능입니다"))
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct WindowsVersionInfo {
        pub os_name: String,
        pub product_name: String,
        pub display_version: String,
        pub build_number: String,
        pub ubr: u32,
        pub install_date: String,
        pub install_language: String,
        pub architecture: String,
    }

    pub fn get_windows_version() -> WindowsVersionInfo {
        WindowsVersionInfo {
            os_name: "Non-Windows".to_string(),
            product_name: String::new(),
            display_version: String::new(),
            build_number: "0".to_string(),
            ubr: 0,
            install_date: String::new(),
            install_language: String::new(),
            architecture: String::new(),
        }
    }
}
