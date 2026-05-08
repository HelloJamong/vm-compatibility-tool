# VBS / 코어 격리 레지스트리 가이드

이 문서는 VM 호환성 도구가 점검하거나 비활성화하는 **VBS / 코어 격리 관련 레지스트리**를
운영자 관점에서 정리한 참고 문서입니다.

## 1. 적용 원칙

- 값이 **실제로 존재하고 활성 상태일 때만** `0`으로 변경합니다.
- 값이 없으면 새로 만들지 않습니다.
- `CurrentAndControlSet001` 항목은 `HKLM\SYSTEM\CurrentControlSet\...` 와 `HKLM\SYSTEM\ControlSet001\...` 양쪽에 함께 반영됩니다.
- 조직 정책, MDM, Windows Hello for Business(WHfB) 영향으로 재부팅 후 값이 다시 활성화될 수 있습니다.
- 조직 장치(Azure AD / MDM 관리)에서는 `HKLM\SOFTWARE\Policies\` 경로 항목을 건너뛰어 GPO 재적용 충돌을 방지합니다.

---

## 2. 자동 조치 대상 레지스트리 (DisableWrite)

아래 항목은 점검 결과에서 활성 상태가 감지되면 자동으로 `0` 으로 변경됩니다.

### VBS (가상화 기반 보안)

| 레지스트리 전체 경로 | 값 이름 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `EnableVirtualizationBasedSecurity` | VBS 전체 활성화 스위치 | **높음** — 메모리 무결성, Credential Guard 등 상위 보호 기능 기반이 약화될 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `RequirePlatformSecurityFeatures` | TPM / 보안 부팅 등 플랫폼 보안 요구 강제 | **중간** — 하드웨어 기반 보안 강제 수준이 낮아질 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `Mandatory` | VBS 강제 적용 상태 | **중간** — 보안 기능 강제성이 낮아질 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard` | `Enabled` | Credential Guard 활성화 여부 | **높음** — LSASS 자격 증명 보호 수준이 낮아질 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `LsaCfgFlags` | LSA 보호 / 격리 관련 플래그 | **높음** — 인증 정보 보호 수준 저하 가능성이 있습니다. |

> `CurrentControlSet` 기반 VBS 항목은 `ControlSet001`에도 함께 반영됩니다.

### VBS 정책 (SOFTWARE\Policies)

| 레지스트리 전체 경로 | 값 이름 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `EnableVirtualizationBasedSecurity` | 정책 기반 VBS 활성화 | **높음** — 조직 보안 정책을 우회하는 결과가 될 수 있습니다. |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `RequirePlatformSecurityFeatures` | 정책 기반 플랫폼 보안 요구 | **중간** — 정책 강제 수준이 완화될 수 있습니다. |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `LsaCfgFlags` | 정책 기반 LSA 보호 | **높음** — 조직 정책 기반 자격 증명 보호가 약화될 수 있습니다. |

### 코어 격리 (HVCI)

| 레지스트리 전체 경로 | 값 이름 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` | `Enabled` | HVCI(메모리 무결성) 활성화 여부 | **높음** — 커널 코드 무결성 보호가 약화될 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` | `Locked` | HVCI 잠금 상태 | **중간** — 코어 격리 정책 변경이 쉬워질 수 있습니다. |

> `CurrentControlSet` 기반 코어 격리 항목은 `ControlSet001`에도 함께 반영됩니다.

### 코어 격리 정책 (SOFTWARE\Policies)

| 레지스트리 전체 경로 | 값 이름 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `HypervisorEnforcedCodeIntegrity` | 정책 기반 HVCI 활성화 | **높음** — 조직 정책 기반 메모리 무결성이 약화될 수 있습니다. |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `HVCIEnabled` | 정책 기반 HVCI 플래그 | **높음** — 정책 강제 보호 수준이 낮아질 수 있습니다. |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `HVCIMATRequired` | MAT 요구 조건 강제 | **중간** — 드라이버 / 플랫폼 요구 강제가 완화될 수 있습니다. |

---

## 3. 자동 조치 제외, 수동 선택 가능 항목 (ExcludedLegacy)

아래 항목은 기본 자동 조치에는 포함되지 않으며, 점검 결과상 실제로 활성 상태일 때만
**실행 전 추가 체크박스**로 선택할 수 있습니다.

| 레지스트리 전체 경로 | 값 이름 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\CI\Config` | `VulnerableDriverBlocklistEnable` | 취약 드라이버 차단 목록 (BYOVD 방어) | **높음** — HVCI와 독립적인 BYOVD 방어 메커니즘입니다. 비활성화 시 알려진 취약 드라이버 로딩 위험이 증가합니다. VM 호환성에 영향 없으나 보안 위험으로 기본 조치 제외. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `EnableSecureLaunch` | Secure Launch (DRTM) | **중간~높음** — DRTM 기반 부팅 무결성에 영향이 있을 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `RequireMicrosoftSignedBootChain` | Microsoft 서명 부트 체인 강제 | **중간~높음** — 부팅 무결성 보장 수준이 낮아질 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SystemGuard` | `Enabled` | System Guard 보호 기능 | **중간~높음** — 부팅 / 런타임 무결성 보호가 감소할 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\SecureBiometrics` | `Enabled` | 보안 생체 인증 보호 | **중간** — Windows Hello / 생체 인증 보안 경로에 영향이 있을 수 있습니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `LsaCfgFlagsDefault` | LSA 기본 보호 기본값 | **중간** — 기본 보안 구성 해석이 달라질 수 있습니다. |
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` | `ConfigureSystemGuardLaunch` | System Guard 정책 시작 추가설정 | **중간~높음** — 조직 정책 기반 보호가 약화될 수 있습니다. |

---

## 4. 읽기 전용 / 점검 전용 항목 (InspectOnly)

아래 항목은 **자동 조치 및 수동 선택 대상이 아닙니다.** 점검 결과 해석에만 사용됩니다.

| 레지스트리 전체 경로 | 값 이름 | 용도 | 제외 사유 |
|---|---|---|---|
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` | `Locked` | VBS UEFI 잠금 상태 확인 | UEFI 펌웨어 잠금은 레지스트리 쓰기로 해제되지 않습니다. 값을 `0`으로 써도 실제 잠금은 유지되므로 허위 성공을 방지하기 위해 점검 전용으로 처리합니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` | `WasEnabledBy` | HVCI 활성화 주체 파악 | 상태 해석용 참고값입니다. "누가 HVCI를 활성화했는지"를 파악하는 용도입니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `RunAsPPL` | LSA 보호 프로세스 활성화 여부 | 직접 비활성화 시 자격 증명 보호가 해제될 위험이 있어 점검만 수행합니다. |
| `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` | `RunAsPPLBoot` | 부팅 시 LSA 보호 적용 여부 | 부팅 시 LSA 보호 활성화 여부 확인용 — 점검 전용입니다. |

### Windows Hello for Business

- 선택형 레지스트리 조치가 아니라 **선행 확인 항목**입니다.
- WHfB, 회사/학교 계정 연결, 조직 정책, MDM 영향이 있으면
  VBS 관련 설정이 재부팅 후 다시 복구될 수 있습니다.
- 이 경우 도구는 먼저 WHfB / 조직 관리 상태를 점검하도록 안내합니다.

---

## 5. 운영 권장사항

- 운영 환경에 적용하기 전 **테스트 장비 또는 스냅샷 환경**에서 먼저 확인하세요.
- 조직 관리 PC에서는 로컬 조치보다 **GPO / MDM 정책 상태 확인**이 우선일 수 있습니다.
- 조직 장치 감지 시 `SOFTWARE\Policies\` 경로 항목은 자동으로 건너뜁니다 — GPO 재적용 충돌 방지 목적입니다.
- 조치 후에는 반드시 **재부팅 후 재점검**으로 실제 반영 여부를 확인하세요.
