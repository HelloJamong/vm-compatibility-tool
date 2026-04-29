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

    deduped.sort_by_key(|item| item.name.to_lowercase());
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
$startAppNames = @{}
Get-StartApps -ErrorAction SilentlyContinue | ForEach-Object {
  if ($_.AppID -match '^(.+?)!') {
    $familyName = $Matches[1]
    if (-not $startAppNames.ContainsKey($familyName)) {
      $startAppNames[$familyName] = $_.Name
    }
  }
}

function Test-GuidName([string]$value) {
  if ([string]::IsNullOrWhiteSpace($value)) {
    return $false
  }
  $guid = [Guid]::Empty
  return [Guid]::TryParse($value, [ref]$guid)
}

function Test-InternalAppxPackage($package) {
  $name = [string]$package.Name
  $publisher = [string]$package.Publisher

  if ($package.IsFramework -or $package.IsResourcePackage -or $package.NonRemovable) {
    return $true
  }
  if ($package.SignatureKind -eq 'System') {
    return $true
  }
  if (Test-GuidName $name) {
    return $true
  }
  if ($publisher -like 'CN=Microsoft Windows,*') {
    return $true
  }
  if ($name -match '^MicrosoftWindows\.') {
    return $true
  }
  if ($name -in @(
    'Microsoft.AAD.BrokerPlugin',
    'Microsoft.AccountsControl',
    'Microsoft.BioEnrollment',
    'Microsoft.CredDialogHost',
    'Microsoft.LockApp',
    'Microsoft.Windows.Apprep.ChxApp',
    'Microsoft.Windows.AssignedAccessLockApp',
    'Microsoft.Windows.CapturePicker',
    'Microsoft.Windows.CloudExperienceHost',
    'Microsoft.Windows.ContentDeliveryManager',
    'Microsoft.Windows.OOBENetworkCaptivePortal',
    'Microsoft.Windows.OOBENetworkConnectionFlow',
    'Microsoft.Windows.ParentalControls',
    'Microsoft.Windows.PeopleExperienceHost',
    'Microsoft.Windows.PinningConfirmationDialog',
    'Microsoft.Windows.PrintQueueActionCenter',
    'Microsoft.Windows.SecureAssessmentBrowser',
    'Microsoft.Windows.ShellExperienceHost',
    'Microsoft.Windows.StartMenuExperienceHost',
    'Microsoft.Windows.XGpuEjectDialog',
    'NcsiUwpApp',
    'Windows.CBSPreview',
    'Windows.PrintDialog',
    'windows.immersivecontrolpanel'
  )) {
    return $true
  }

  return $false
}

function Get-ManifestText($package, [string]$propertyName) {
  try {
    $manifest = Get-AppxPackageManifest -Package $package.PackageFullName -ErrorAction Stop
    $value = $manifest.Package.Properties.$propertyName
    if ($value -and -not ([string]$value).StartsWith('ms-resource:')) {
      return [string]$value
    }
  } catch {
  }
  return ''
}

$packages = Get-AppxPackage |
  Where-Object { -not (Test-InternalAppxPackage $_) } |
  Select-Object `
    @{Name='name';Expression={
      $startName = $startAppNames[$_.PackageFamilyName]
      if (-not [string]::IsNullOrWhiteSpace($startName)) {
        $startName
      } else {
        $manifestName = Get-ManifestText $_ 'DisplayName'
        if (-not [string]::IsNullOrWhiteSpace($manifestName)) { $manifestName } else { $_.Name }
      }
    }}, `
    @{Name='publisher';Expression={
      $manifestPublisher = Get-ManifestText $_ 'PublisherDisplayName'
      if (-not [string]::IsNullOrWhiteSpace($manifestPublisher)) { $manifestPublisher } else { $_.Publisher }
    }}, `
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
            if should_skip_appx_name(&name) {
                return None;
            }
            Some(InstalledProgramItem::new(
                &name,
                &normalize_appx_publisher(&item.publisher.unwrap_or_default()),
                &item.install_date.unwrap_or_default(),
            ))
        })
        .collect()
}

#[cfg(not(windows))]
fn collect_appx_packages() -> Vec<InstalledProgramItem> {
    Vec::new()
}

fn should_skip_appx_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || is_guid_like(trimmed) {
        return true;
    }

    if trimmed.starts_with("MicrosoftWindows.") {
        return true;
    }

    matches!(
        trimmed,
        "Microsoft.AAD.BrokerPlugin"
            | "Microsoft.AccountsControl"
            | "Microsoft.BioEnrollment"
            | "Microsoft.CredDialogHost"
            | "Microsoft.LockApp"
            | "Microsoft.Windows.Apprep.ChxApp"
            | "Microsoft.Windows.AssignedAccessLockApp"
            | "Microsoft.Windows.CapturePicker"
            | "Microsoft.Windows.CloudExperienceHost"
            | "Microsoft.Windows.ContentDeliveryManager"
            | "Microsoft.Windows.OOBENetworkCaptivePortal"
            | "Microsoft.Windows.OOBENetworkConnectionFlow"
            | "Microsoft.Windows.ParentalControls"
            | "Microsoft.Windows.PeopleExperienceHost"
            | "Microsoft.Windows.PinningConfirmationDialog"
            | "Microsoft.Windows.PrintQueueActionCenter"
            | "Microsoft.Windows.SecureAssessmentBrowser"
            | "Microsoft.Windows.ShellExperienceHost"
            | "Microsoft.Windows.StartMenuExperienceHost"
            | "Microsoft.Windows.XGpuEjectDialog"
            | "NcsiUwpApp"
            | "Windows.CBSPreview"
            | "Windows.PrintDialog"
            | "windows.immersivecontrolpanel"
    )
}

fn is_guid_like(value: &str) -> bool {
    let parts = value.split('-').collect::<Vec<_>>();
    if parts.len() != 5 {
        return false;
    }

    let expected_lengths = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(expected_lengths)
        .all(|(part, len)| part.len() == len && part.chars().all(|ch| ch.is_ascii_hexdigit()))
}

fn normalize_appx_publisher(publisher: &str) -> String {
    let trimmed = publisher.trim();
    if trimmed.is_empty() || !trimmed.contains('=') {
        return trimmed.to_string();
    }

    let organization = distinguished_name_value(trimmed, "O")
        .filter(|value| !is_guid_like(value))
        .or_else(|| distinguished_name_value(trimmed, "CN").filter(|value| !is_guid_like(value)));

    organization.map(ToString::to_string).unwrap_or_default()
}

fn distinguished_name_value<'a>(publisher: &'a str, key: &str) -> Option<&'a str> {
    publisher.split(',').find_map(|part| {
        let part = part.trim();
        let (current_key, value) = part.split_once('=')?;
        if current_key.trim() != key {
            return None;
        }
        Some(value.trim().trim_matches('"'))
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_appx_publisher, normalize_install_date, should_skip_appx_name};

    #[test]
    fn normalizes_registry_install_date() {
        assert_eq!(normalize_install_date("20260427"), "2026-04-27");
    }

    #[test]
    fn leaves_unknown_install_date_as_is() {
        assert_eq!(normalize_install_date(""), "");
        assert_eq!(normalize_install_date("2026-04-27"), "2026-04-27");
    }

    #[test]
    fn skips_internal_appx_package_names() {
        assert!(should_skip_appx_name(
            "1527c705-839a-4832-9118-54d4Bd6a0c89"
        ));
        assert!(should_skip_appx_name("MicrosoftWindows.Client.CBS"));
        assert!(should_skip_appx_name(
            "Microsoft.Windows.ShellExperienceHost"
        ));
        assert!(!should_skip_appx_name("Microsoft.WindowsCalculator"));
        assert!(!should_skip_appx_name("Notepad++"));
    }

    #[test]
    fn normalizes_appx_distinguished_name_publishers() {
        assert_eq!(
            normalize_appx_publisher(
                "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US"
            ),
            "Microsoft Corporation"
        );
        assert_eq!(
            normalize_appx_publisher("CN=\"Notepad++\", O=\"Notepad++\", L=Saint Cloud"),
            "Notepad++"
        );
        assert_eq!(
            normalize_appx_publisher("CN=33F0F141-36F3-4EC2-A77D-51B53D0BA0E4"),
            ""
        );
    }
}
