use crate::models::virtualization::DisableGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryAction {
    DisableWrite,
    InspectOnly,
    ExcludedLegacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryHiveSet {
    CurrentOnly,
    CurrentAndControlSet001,
    PoliciesOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct RegistryManifestEntry {
    pub id: &'static str,
    pub path: &'static str,
    pub value_name: &'static str,
    pub target_value: Option<u32>,
    pub disable_group: DisableGroup,
    pub action: RegistryAction,
    pub hive_set: RegistryHiveSet,
    pub label: &'static str,
    pub rationale: &'static str,
}

#[derive(Debug, Clone)]
pub struct ResolvedRegistryManifestEntry {
    pub id: &'static str,
    pub path: String,
    pub value_name: &'static str,
    pub target_value: Option<u32>,
    pub disable_group: DisableGroup,
    pub action: RegistryAction,
    pub label: &'static str,
    pub rationale: &'static str,
}

const MANIFEST: &[RegistryManifestEntry] = &[
    RegistryManifestEntry {
        id: "vbs.enable_virtualization_based_security",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard",
        value_name: "EnableVirtualizationBasedSecurity",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "VBS (가상화 기반 보안)",
        rationale: "VBS 전체 활성화 스위치",
    },
    RegistryManifestEntry {
        id: "vbs.require_platform_security_features",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard",
        value_name: "RequirePlatformSecurityFeatures",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "플랫폼 보안 기능 요구",
        rationale: "VBS가 추가 플랫폼 보안 기능을 요구하지 않도록 설정",
    },
    RegistryManifestEntry {
        id: "vbs.locked",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard",
        value_name: "Locked",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "VBS 잠금",
        rationale: "DeviceGuard 잠금 상태 해제",
    },
    RegistryManifestEntry {
        id: "vbs.mandatory",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard",
        value_name: "Mandatory",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "VBS 강제",
        rationale: "DeviceGuard 강제 적용 상태 해제",
    },
    RegistryManifestEntry {
        id: "core_isolation.hvci_enabled",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
        value_name: "Enabled",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "HVCI (코어 격리)",
        rationale: "하이퍼바이저 코드 무결성 비활성화",
    },
    RegistryManifestEntry {
        id: "core_isolation.hvci_locked",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
        value_name: "Locked",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "HVCI 잠금",
        rationale: "HVCI 잠금 상태 해제",
    },
    RegistryManifestEntry {
        id: "core_isolation.hvci_was_enabled_by",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity",
        value_name: "WasEnabledBy",
        target_value: None,
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::InspectOnly,
        hive_set: RegistryHiveSet::CurrentOnly,
        label: "HVCI 활성화 주체",
        rationale: "상태/UI 해석용 값으로 점검만 수행",
    },
    RegistryManifestEntry {
        id: "vbs.credential_guard_enabled",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard",
        value_name: "Enabled",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "CredentialGuard",
        rationale: "Credential Guard 비활성화",
    },
    RegistryManifestEntry {
        id: "vbs.lsa_cfg_flags",
        path: r"SYSTEM\CurrentControlSet\Control\Lsa",
        value_name: "LsaCfgFlags",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "LSA 보호",
        rationale: "LSA 보호 플래그 비활성화",
    },
    RegistryManifestEntry {
        id: "vbs.policy_enable_virtualization_based_security",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "EnableVirtualizationBasedSecurity",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "VBS 정책",
        rationale: "정책 기반 VBS 활성화 해제",
    },
    RegistryManifestEntry {
        id: "vbs.policy_require_platform_security_features",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "RequirePlatformSecurityFeatures",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "VBS 정책 플랫폼 보안",
        rationale: "정책 기반 플랫폼 보안 요구 해제",
    },
    RegistryManifestEntry {
        id: "vbs.policy_lsa_cfg_flags",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "LsaCfgFlags",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "LSA 정책",
        rationale: "정책 기반 LSA 보호 비활성화",
    },
    RegistryManifestEntry {
        id: "core_isolation.policy_hvci",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "HypervisorEnforcedCodeIntegrity",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "HVCI 정책",
        rationale: "정책 기반 HVCI 비활성화",
    },
    RegistryManifestEntry {
        id: "core_isolation.policy_hvci_enabled",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "HVCIEnabled",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "HVCI 활성화 정책",
        rationale: "정책 기반 HVCI 활성 플래그 비활성화",
    },
    RegistryManifestEntry {
        id: "core_isolation.policy_hvci_mat_required",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "HVCIMATRequired",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "HVCI MAT 요구",
        rationale: "정책 기반 MAT 요구 해제",
    },
    RegistryManifestEntry {
        id: "core_isolation.vulnerable_driver_blocklist",
        path: r"SYSTEM\CurrentControlSet\Control\CI\Config",
        value_name: "VulnerableDriverBlocklistEnable",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::DisableWrite,
        hive_set: RegistryHiveSet::CurrentOnly,
        label: "취약 드라이버 차단",
        rationale: "코어 격리 관련 차단 정책 해제",
    },
    RegistryManifestEntry {
        id: "legacy.require_microsoft_signed_boot_chain",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard",
        value_name: "RequireMicrosoftSignedBootChain",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::ExcludedLegacy,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "MS 서명 부트 체인 요구",
        rationale: "효과 재검증 전까지 기본 write 대상에서 제외",
    },
    RegistryManifestEntry {
        id: "legacy.system_guard_enabled",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard",
        value_name: "Enabled",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::ExcludedLegacy,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "SystemGuard",
        rationale: "기본 비활성화 세트에서 보류",
    },
    RegistryManifestEntry {
        id: "legacy.secure_biometrics_enabled",
        path: r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SecureBiometrics",
        value_name: "Enabled",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::ExcludedLegacy,
        hive_set: RegistryHiveSet::CurrentOnly,
        label: "SecureBiometrics",
        rationale: "기본 비활성화 세트에서 보류",
    },
    RegistryManifestEntry {
        id: "legacy.lsa_cfg_flags_default",
        path: r"SYSTEM\CurrentControlSet\Control\Lsa",
        value_name: "LsaCfgFlagsDefault",
        target_value: Some(0),
        disable_group: DisableGroup::Vbs,
        action: RegistryAction::ExcludedLegacy,
        hive_set: RegistryHiveSet::CurrentAndControlSet001,
        label: "LSA 기본 구성",
        rationale: "핵심 플래그가 아니므로 기본 세트에서 제외",
    },
    RegistryManifestEntry {
        id: "legacy.configure_system_guard_launch",
        path: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
        value_name: "ConfigureSystemGuardLaunch",
        target_value: Some(0),
        disable_group: DisableGroup::CoreIsolation,
        action: RegistryAction::ExcludedLegacy,
        hive_set: RegistryHiveSet::PoliciesOnly,
        label: "SystemGuard 시작 정책",
        rationale: "효과 재검증 전까지 보류",
    },
];

pub fn all_manifest_entries() -> &'static [RegistryManifestEntry] {
    MANIFEST
}

pub fn inspect_entries() -> impl Iterator<Item = &'static RegistryManifestEntry> {
    MANIFEST
        .iter()
        .filter(|entry| entry.action != RegistryAction::ExcludedLegacy)
}

pub fn disable_write_entries(group: DisableGroup) -> Vec<ResolvedRegistryManifestEntry> {
    MANIFEST
        .iter()
        .filter(|entry| {
            entry.action == RegistryAction::DisableWrite && entry.disable_group == group
        })
        .flat_map(resolve_entry_paths)
        .collect()
}

pub fn resolve_entry_paths(entry: &RegistryManifestEntry) -> Vec<ResolvedRegistryManifestEntry> {
    let paths = match entry.hive_set {
        RegistryHiveSet::CurrentOnly | RegistryHiveSet::PoliciesOnly => {
            vec![entry.path.to_string()]
        }
        RegistryHiveSet::CurrentAndControlSet001 => vec![
            entry.path.to_string(),
            entry.path.replace(r"CurrentControlSet", r"ControlSet001"),
        ],
    };

    paths
        .into_iter()
        .map(|path| ResolvedRegistryManifestEntry {
            id: entry.id,
            path,
            value_name: entry.value_name,
            target_value: entry.target_value,
            disable_group: entry.disable_group,
            action: entry.action,
            label: entry.label,
            rationale: entry.rationale,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        all_manifest_entries, disable_write_entries, resolve_entry_paths, RegistryAction,
        RegistryHiveSet,
    };
    use crate::models::virtualization::DisableGroup;

    #[test]
    fn inspect_only_entry_keeps_current_controlset_scope() {
        let entry = all_manifest_entries()
            .iter()
            .find(|entry| entry.id == "core_isolation.hvci_was_enabled_by")
            .expect("manifest entry exists");

        assert_eq!(entry.action, RegistryAction::InspectOnly);
        assert_eq!(entry.hive_set, RegistryHiveSet::CurrentOnly);
        assert_eq!(resolve_entry_paths(entry).len(), 1);
    }

    #[test]
    fn mirrored_disable_entries_expand_for_controlset001() {
        let entries = disable_write_entries(DisableGroup::Vbs);

        assert!(entries.iter().any(|entry| {
            entry.path == r"SYSTEM\CurrentControlSet\Control\DeviceGuard"
                && entry.value_name == "EnableVirtualizationBasedSecurity"
        }));
        assert!(entries.iter().any(|entry| {
            entry.path == r"SYSTEM\ControlSet001\Control\DeviceGuard"
                && entry.value_name == "EnableVirtualizationBasedSecurity"
        }));
    }

    #[test]
    fn excluded_legacy_entries_are_not_inspected() {
        let inspectable_ids: Vec<_> = super::inspect_entries().map(|entry| entry.id).collect();

        assert!(!inspectable_ids.contains(&"legacy.system_guard_enabled"));
        assert!(!inspectable_ids.contains(&"legacy.configure_system_guard_launch"));
    }
}
