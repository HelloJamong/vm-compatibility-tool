use crate::models::installed_program::InstalledProgramItem;
use std::collections::HashSet;

pub fn collect_installed_programs() -> Vec<InstalledProgramItem> {
    let mut items = collect_win32_uninstall_entries();
    items.extend(collect_appx_packages());
    dedupe_and_sort(items)
}

fn dedupe_and_sort(items: Vec<InstalledProgramItem>) -> Vec<InstalledProgramItem> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for item in items {
        let normalized_name = item.name.trim().to_lowercase();
        if normalized_name.is_empty() {
            continue;
        }

        let key = format!(
            "{}|{}",
            normalized_name,
            item.publisher.trim().to_lowercase()
        );
        if seen.insert(key) {
            deduped.push(InstalledProgramItem {
                name: item.name.trim().to_string(),
                publisher: item.publisher.trim().to_string(),
                install_date: normalize_install_date(&item.install_date),
            });
        }
    }

    deduped.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    deduped
}

fn normalize_install_date(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() == 8 && trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        format!("{}-{}-{}", &trimmed[0..4], &trimmed[4..6], &trimmed[6..8])
    } else {
        trimmed.to_string()
    }
}

#[cfg(windows)]
fn collect_win32_uninstall_entries() -> Vec<InstalledProgramItem> {
    use winreg::{enums::*, RegKey};

    let roots = [
        RegKey::predef(HKEY_LOCAL_MACHINE),
        RegKey::predef(HKEY_CURRENT_USER),
    ];
    let uninstall_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    let mut items = Vec::new();
    for root in roots {
        for path in uninstall_paths {
            let Ok(uninstall_key) = root.open_subkey(path) else {
                continue;
            };

            for subkey_name in uninstall_key.enum_keys().filter_map(Result::ok) {
                let Ok(app_key) = uninstall_key.open_subkey(&subkey_name) else {
                    continue;
                };

                let system_component = app_key.get_value::<u32, _>("SystemComponent").unwrap_or(0);
                if system_component == 1 {
                    continue;
                }

                if app_key.get_value::<String, _>("ParentKeyName").is_ok() {
                    continue;
                }

                let name = app_key
                    .get_value::<String, _>("DisplayName")
                    .unwrap_or_default();
                if name.trim().is_empty() || is_update_entry(&app_key) {
                    continue;
                }

                let publisher = app_key
                    .get_value::<String, _>("Publisher")
                    .unwrap_or_default();
                let install_date = app_key
                    .get_value::<String, _>("InstallDate")
                    .unwrap_or_default();

                items.push(InstalledProgramItem::new(&name, &publisher, &install_date));
            }
        }
    }

    items
}

#[cfg(windows)]
fn is_update_entry(app_key: &winreg::RegKey) -> bool {
    let release_type = app_key
        .get_value::<String, _>("ReleaseType")
        .unwrap_or_default()
        .to_lowercase();

    matches!(
        release_type.as_str(),
        "hotfix" | "security update" | "update rollup" | "service pack"
    )
}

#[cfg(not(windows))]
fn collect_win32_uninstall_entries() -> Vec<InstalledProgramItem> {
    Vec::new()
}

#[cfg(windows)]
fn collect_appx_packages() -> Vec<InstalledProgramItem> {
    use crate::services::process_service;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct AppxPackageJson {
        name: Option<String>,
        publisher: Option<String>,
        install_date: Option<String>,
    }

    let script = r#"
$packages = Get-AppxPackage -AllUsers |
  Where-Object { -not $_.IsFramework } |
  Select-Object `
    @{Name='name';Expression={$_.Name}}, `
    @{Name='publisher';Expression={$_.Publisher}}, `
    @{Name='install_date';Expression={ if ($_.InstallDate) { $_.InstallDate.ToString('yyyy-MM-dd') } else { '' } }}
$packages | ConvertTo-Json -Compress
"#;

    let result = process_service::run_powershell(script);
    if !result.success || result.stdout.trim().is_empty() {
        return Vec::new();
    }

    let stdout = result.stdout.trim();
    let parsed_items: Vec<AppxPackageJson> = if stdout.starts_with('[') {
        serde_json::from_str(stdout).unwrap_or_default()
    } else {
        serde_json::from_str::<AppxPackageJson>(stdout)
            .map(|item| vec![item])
            .unwrap_or_default()
    };

    parsed_items
        .into_iter()
        .filter_map(|item| {
            let name = item.name.unwrap_or_default();
            if name.trim().is_empty() {
                return None;
            }
            Some(InstalledProgramItem::new(
                &name,
                &item.publisher.unwrap_or_default(),
                &item.install_date.unwrap_or_default(),
            ))
        })
        .collect()
}

#[cfg(not(windows))]
fn collect_appx_packages() -> Vec<InstalledProgramItem> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::normalize_install_date;

    #[test]
    fn normalizes_registry_install_date() {
        assert_eq!(normalize_install_date("20260427"), "2026-04-27");
    }

    #[test]
    fn leaves_unknown_install_date_as_is() {
        assert_eq!(normalize_install_date(""), "");
        assert_eq!(normalize_install_date("2026-04-27"), "2026-04-27");
    }
}
