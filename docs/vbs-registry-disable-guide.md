# VBS / 코어 격리 레지스트리 가이드

이 문서는 VM 호환성 도구가 점검하거나 비활성화하는 **VBS / 코어 격리 관련 레지스트리**를
운영자 관점에서 정리한 참고 문서입니다.

## 1. 적용 원칙

- 값이 **실제로 존재하고 활성 상태일 때만** `0`으로 변경합니다.
- 값이 없으면 새로 만들지 않습니다.
- 일부 항목은 `CurrentControlSet`과 `ControlSet001`에 함께 반영됩니다.
- 조직 정책, MDM, Windows Hello for Business(WHfB) 영향으로 재부팅 후 값이 다시 활성화될 수 있습니다.

---

## 2. 자동 조치 대상 레지스트리

| 구분 | 레지스트리 값 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| VBS | `DeviceGuard\\EnableVirtualizationBasedSecurity` | VBS 전체 활성화 스위치 | **높음** — 메모리 무결성, Credential Guard 등 상위 보호 기능 기반이 약화될 수 있습니다. |
| VBS | `DeviceGuard\\RequirePlatformSecurityFeatures` | TPM / 보안 부팅 등 플랫폼 보안 요구 강제 | **중간** — 하드웨어 기반 보안 강제 수준이 낮아질 수 있습니다. |
| VBS | `DeviceGuard\\Locked` | VBS / Device Guard 잠금 상태 | **중간** — 정책 잠금이 풀려 재구성이 쉬워질 수 있습니다. |
| VBS | `DeviceGuard\\Mandatory` | VBS 강제 적용 상태 | **중간** — 보안 기능 강제성이 낮아질 수 있습니다. |
| VBS | `Scenarios\\CredentialGuard\\Enabled` | Credential Guard 활성화 여부 | **높음** — LSASS 자격 증명 보호 수준이 낮아질 수 있습니다. |
| VBS | `Lsa\\LsaCfgFlags` | LSA 보호 / 격리 관련 플래그 | **높음** — 인증 정보 보호 수준 저하 가능성이 있습니다. |
| VBS 정책 | `Policies\\...\\EnableVirtualizationBasedSecurity` | 정책 기반 VBS 활성화 | **높음** — 조직 보안 정책을 우회하는 결과가 될 수 있습니다. |
| VBS 정책 | `Policies\\...\\RequirePlatformSecurityFeatures` | 정책 기반 플랫폼 보안 요구 | **중간** — 정책 강제 수준이 완화될 수 있습니다. |
| VBS 정책 | `Policies\\...\\LsaCfgFlags` | 정책 기반 LSA 보호 | **높음** — 조직 정책 기반 자격 증명 보호가 약화될 수 있습니다. |
| 코어 격리 | `Scenarios\\HypervisorEnforcedCodeIntegrity\\Enabled` | HVCI(메모리 무결성) 활성화 여부 | **높음** — 커널 코드 무결성 보호가 약화될 수 있습니다. |
| 코어 격리 | `Scenarios\\HypervisorEnforcedCodeIntegrity\\Locked` | HVCI 잠금 상태 | **중간** — 코어 격리 정책 변경이 쉬워질 수 있습니다. |
| 코어 격리 정책 | `Policies\\...\\HypervisorEnforcedCodeIntegrity` | 정책 기반 HVCI 활성화 | **높음** — 조직 정책 기반 메모리 무결성이 약화될 수 있습니다. |
| 코어 격리 정책 | `Policies\\...\\HVCIEnabled` | 정책 기반 HVCI 플래그 | **높음** — 정책 강제 보호 수준이 낮아질 수 있습니다. |
| 코어 격리 정책 | `Policies\\...\\HVCIMATRequired` | MAT 요구 조건 강제 | **중간** — 드라이버 / 플랫폼 요구 강제가 완화될 수 있습니다. |
| 코어 격리 | `CI\\Config\\VulnerableDriverBlocklistEnable` | 취약 드라이버 차단 목록 활성화 | **높음** — 알려진 취약 드라이버 로딩 위험이 증가할 수 있습니다. |

> 참고: `CurrentControlSet` 기반 주요 항목은 필요 시 `ControlSet001`에도 함께 적용됩니다.

---

## 3. 자동 조치 제외, 수동 선택 가능 항목

아래 항목은 기본 자동 조치에는 포함되지 않으며, 점검 결과상 실제로 활성 상태일 때만
**실행 전 추가 체크박스**로 선택할 수 있습니다.

| 레지스트리 값 | 역할 | 비활성화 시 위험도 |
|---|---|---|
| `DeviceGuard\\RequireMicrosoftSignedBootChain` | Microsoft 서명 부트 체인 강제 | **중간~높음** — 부팅 무결성 보장 수준이 낮아질 수 있습니다. |
| `Scenarios\\SystemGuard\\Enabled` | System Guard 보호 기능 | **중간~높음** — 부팅 / 런타임 무결성 보호가 감소할 수 있습니다. |
| `Scenarios\\SecureBiometrics\\Enabled` | 보안 생체 인증 보호 | **중간** — Windows Hello / 생체 인증 보안 경로에 영향이 있을 수 있습니다. |
| `Lsa\\LsaCfgFlagsDefault` | LSA 기본 보호 기본값 | **중간** — 기본 보안 구성 해석이 달라질 수 있습니다. |
| `Policies\\...\\ConfigureSystemGuardLaunch` | System Guard 정책 시작 추가설정 | **중간~높음** — 조직 정책 기반 보호가 약화될 수 있습니다. |

---

## 4. 읽기 전용 / 선행 확인 항목

### `HVCI WasEnabledBy`
- 상태 해석용 참고값입니다.
- 자동 조치 / 수동 선택 조치 대상이 아닙니다.
- “누가 HVCI를 활성화했는지”를 파악하는 용도로만 사용합니다.

### Windows Hello for Business
- 선택형 레지스트리 조치가 아니라 **선행 확인 항목**입니다.
- WHfB, 회사/학교 계정 연결, 조직 정책, MDM 영향이 있으면
  VBS 관련 설정이 재부팅 후 다시 복구될 수 있습니다.
- 이 경우 도구는 먼저 WHfB / 조직 관리 상태를 점검하도록 안내합니다.

---

## 5. 운영 권장사항

- 운영 환경에 적용하기 전 **테스트 장비 또는 스냅샷 환경**에서 먼저 확인하세요.
- 조직 관리 PC에서는 로컬 조치보다 **GPO / MDM 정책 상태 확인**이 우선일 수 있습니다.
- 조치 후에는 반드시 **재부팅 후 재점검**으로 실제 반영 여부를 확인하세요.
