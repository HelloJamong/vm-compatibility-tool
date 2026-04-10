# VM Compatibility Tool

Windows 환경에서 **VM 호환성 점검**과 **Hyper-V / WSL / VBS / 코어 격리 비활성화**를 지원하는
관리자 권한 기반 데스크톱 도구입니다.

현재 프로젝트는 **Tauri v2 + Rust + Svelte 5** 기반으로 동작합니다.
기존 WPF/C# 구현은 마이그레이션 과정에서 사용되었으며, 현재 런타임 기준은 Tauri 앱입니다.

## 주요 기능

### 1. 시스템 정보 수집
- 운영체제 버전 / 빌드 / 에디션
- CPU 정보 및 하드웨어 가상화 활성 여부
- 메모리 / 디스크 / 부팅 정보
- 메인보드 / GPU / 전원 계획
- 최근 이벤트 로그 요약

### 2. 가상화 호환성 점검
- Hyper-V 기능 상태
- WSL / VirtualMachinePlatform 상태
- `hypervisorlaunchtype` 상태
- VBS / HVCI / Credential Guard / LSA 관련 레지스트리 상태
- 참고용 legacy 레지스트리 항목 표시

### 3. 선택적 비활성화
가상화 점검 결과를 바탕으로 필요한 항목만 선택적으로 조치합니다.

- Hyper-V 비활성화
- WSL 비활성화
- VBS 관련 레지스트리 비활성화
- 코어 격리 관련 레지스트리 비활성화
- 재부팅 요청

#### 3-1. Hyper-V 비활성화
- 제거 대상 기능
  - `Microsoft-Hyper-V-All`
  - `Microsoft-Hyper-V`
  - `Microsoft-Hyper-V-Tools-All`
  - `Microsoft-Hyper-V-Management-PowerShell`
  - `Microsoft-Hyper-V-Hypervisor`
  - `Microsoft-Hyper-V-Services`
  - `Microsoft-Hyper-V-Management-Clients`
  - `hypervisorlaunchtype off` 적용
- 역할
  - Windows Hyper-V 하이퍼바이저와 관리 도구를 비활성화해 다른 VM/에뮬레이터와의 충돌 가능성을 낮춥니다.
- 위험도: **중간**
  - Hyper-V 기반 VM, Android Emulator, Docker Desktop(설정에 따라), 일부 보안 기능이 동작하지 않을 수 있습니다.

#### 3-2. WSL 비활성화
- 제거 대상 기능
  - `Microsoft-Windows-Subsystem-Linux`
  - `VirtualMachinePlatform`
- 역할
  - WSL/WSL2 실행에 필요한 Windows 기능을 비활성화합니다.
- 위험도: **중간**
  - WSL 배포판 실행이 불가능해지고, WSL2 의존 개발 환경/Docker 연계 구성이 영향을 받을 수 있습니다.

#### 3-3. VBS / 코어 격리 레지스트리 비활성화
- 원칙
  - 값이 실제로 존재하고 활성 상태일 때만 `0`으로 변경합니다.
  - 값이 없으면 새로 만들지 않습니다.
  - 조직 정책/MDM/Windows Hello for Business가 재적용 원인일 수 있어, 일부 환경에서는 재부팅 후 다시 활성화될 수 있습니다.

##### 자동 조치 대상 레지스트리
| 구분 | 레지스트리 값 | 역할 | 비활성화 시 위험도 |
|---|---|---|---|
| VBS | `DeviceGuard\\EnableVirtualizationBasedSecurity` | VBS 전체 활성화 스위치 | **높음** — 메모리 무결성, Credential Guard 등 상위 보호 기능 기반이 약화될 수 있음 |
| VBS | `DeviceGuard\\RequirePlatformSecurityFeatures` | TPM/보안 부팅 등 플랫폼 보안 요구 강제 | **중간** — 하드웨어 기반 보안 강제 수준이 낮아질 수 있음 |
| VBS | `DeviceGuard\\Locked` | VBS/Device Guard 잠금 상태 | **중간** — 정책 잠금이 풀려 재구성이 쉬워짐 |
| VBS | `DeviceGuard\\Mandatory` | VBS 강제 적용 상태 | **중간** — 보안 기능 강제성이 낮아짐 |
| VBS | `Scenarios\\CredentialGuard\\Enabled` | Credential Guard 활성화 여부 | **높음** — LSASS 자격 증명 보호 수준이 낮아질 수 있음 |
| VBS | `Lsa\\LsaCfgFlags` | LSA 보호/격리 관련 플래그 | **높음** — 인증 정보 보호 수준 저하 가능 |
| VBS 정책 | `Policies\\...\\EnableVirtualizationBasedSecurity` | 정책 기반 VBS 활성화 | **높음** — 조직 보안 정책을 우회하는 결과가 될 수 있음 |
| VBS 정책 | `Policies\\...\\RequirePlatformSecurityFeatures` | 정책 기반 플랫폼 보안 요구 | **중간** — 정책 강제 수준 완화 |
| VBS 정책 | `Policies\\...\\LsaCfgFlags` | 정책 기반 LSA 보호 | **높음** — 조직 정책 기반 자격 증명 보호 약화 |
| 코어 격리 | `Scenarios\\HypervisorEnforcedCodeIntegrity\\Enabled` | HVCI(메모리 무결성) 활성화 여부 | **높음** — 커널 코드 무결성 보호 약화 |
| 코어 격리 | `Scenarios\\HypervisorEnforcedCodeIntegrity\\Locked` | HVCI 잠금 상태 | **중간** — 코어 격리 정책 변경이 쉬워짐 |
| 코어 격리 정책 | `Policies\\...\\HypervisorEnforcedCodeIntegrity` | 정책 기반 HVCI 활성화 | **높음** — 조직 정책 기반 메모리 무결성 약화 |
| 코어 격리 정책 | `Policies\\...\\HVCIEnabled` | 정책 기반 HVCI 플래그 | **높음** — 정책 강제 보호 수준 저하 |
| 코어 격리 정책 | `Policies\\...\\HVCIMATRequired` | MAT 요구 조건 강제 | **중간** — 드라이버/플랫폼 요구 강제 완화 |
| 코어 격리 | `CI\\Config\\VulnerableDriverBlocklistEnable` | 취약 드라이버 차단 목록 활성화 | **높음** — 알려진 취약 드라이버 로딩 위험 증가 |

> 참고: `CurrentControlSet` 기반 항목 중 일부는 `ControlSet001`에도 함께 반영합니다.

##### 자동 조치 제외, 수동 선택 가능 항목
아래 항목은 기본 자동 조치에는 포함되지 않으며, 점검 결과상 실제로 활성 상태일 때만 **추가 체크박스**로 선택할 수 있습니다.

| 레지스트리 값 | 역할 | 비활성화 시 위험도 |
|---|---|---|
| `DeviceGuard\\RequireMicrosoftSignedBootChain` | Microsoft 서명 부트 체인 강제 | **중간~높음** — 부팅 무결성 보장 수준 저하 가능 |
| `Scenarios\\SystemGuard\\Enabled` | System Guard 보호 기능 | **중간~높음** — 부팅/런타임 무결성 보호 감소 가능 |
| `Scenarios\\SecureBiometrics\\Enabled` | 보안 생체 인증 보호 | **중간** — Hello/생체 인증 보안 경로에 영향 가능 |
| `Lsa\\LsaCfgFlagsDefault` | LSA 기본 보호 기본값 | **중간** — 기본 보안 구성 해석이 바뀔 수 있음 |
| `Policies\\...\\ConfigureSystemGuardLaunch` | System Guard 정책 시작 설정 | **중간~높음** — 조직 정책 기반 보호 약화 가능 |

##### 읽기 전용/수동 선행조치 항목
- `HVCI WasEnabledBy`
  - 상태 해석용 참고값입니다. 자동/수동 조치 대상이 아닙니다.
- `Windows Hello for Business`
  - 선택형 레지스트리 조치가 아니라 **선행 확인 항목**입니다.
  - WHfB, MDM, 조직 정책이 활성화된 기기는 VBS 설정이 재부팅 후 복구될 수 있습니다.
  - 이 경우 앱은 먼저 WHfB/회사 또는 학교 계정/정책 상태를 확인하도록 안내합니다.

### 4. 레지스트리 정책
이 프로젝트는 다음 정책으로 동작합니다.

- 점검 시 값이 **없으면** `미설정`으로 표시
- 값이 없다고 해서 **새 레지스트리 항목을 생성하지 않음**
- 값이 **실제로 존재하고 활성 상태일 때만** 비활성화 시 `0`으로 변경
- legacy/참고용 항목은 표시만 하고 자동 조치에는 포함하지 않음

### 5. CSV 내보내기
- 시스템 정보 CSV
- 가상화 점검 결과 CSV

---

## 기술 스택

### Frontend
- Svelte 5
- TypeScript
- Vite 6

### Backend
- Rust
- Tauri v2
- `wmi`
- `winreg`
- `windows-sys`

### Build / Packaging
- `npm run tauri build -- --no-bundle` : 포터블 EXE
- `npm run tauri build -- --bundles nsis` : NSIS 인스톨러

---

## 시스템 요구사항
- Windows 10/11 64-bit
- 관리자 권한
- WebView2 사용 가능 환경

---

## 내부 배포 실행 안내

이 도구는 현재 **내부 배포용 실행 파일** 기준으로 사용됩니다.  
Windows 보안 정책 및 Microsoft SmartScreen 상태에 따라, 다운로드한 EXE 실행 시
경고 화면이 표시될 수 있습니다.

### SmartScreen 경고가 보일 때
아래 순서로 실행하세요.

1. EXE 실행
2. `Windows에서 PC를 보호했습니다` 화면이 나오면 **추가 정보** 클릭
3. 게시자/파일명을 다시 확인
4. 내부 배포 파일이 맞다면 **실행** 클릭

### 실행 전 확인 사항
- 배포 경로가 내부 공유 위치 또는 내부 공지된 릴리즈 링크인지 확인
- 파일명이 배포 공지와 일치하는지 확인
- 반드시 **관리자 권한으로 실행**

### 주의
- 이 경고는 현재 배포 방식상 나타날 수 있습니다.
- 일반 사용자용 공개 배포가 아니라 **내부 확인된 배포본**인지 먼저 확인한 뒤 진행하세요.

---

## 로컬 개발

```bash
git clone https://github.com/HelloJamong/vm-compatibility-tool.git
cd vm-compatibility-tool
npm ci
```

### 프론트엔드 타입/구조 점검
```bash
npm run check
```

### 개발 실행
```bash
npm run tauri dev
```

### 포터블 EXE 빌드
```bash
npm run tauri build -- --no-bundle
```

### NSIS 인스톨러 빌드
```bash
npm run tauri build -- --bundles nsis
```

### Windows용 대화형 빌드 스크립트
```bat
build.bat
```

---

## 저장소 구조

```text
src/                       Svelte 앱 진입점 및 UI 컴포넌트
src/components/            UI 패널 / 공통 컴포넌트
src/lib/                   프론트 공용 타입
src-tauri/src/commands/    Tauri command 레이어
src-tauri/src/services/    Windows/WMI/Registry/Process 서비스
src-tauri/src/models/      Rust 직렬화 모델
src-tauri/src/main.rs      관리자 권한 확인 + 앱 진입점
src-tauri/build.rs         Windows manifest / Tauri build 설정
.github/workflows/         beta / release 빌드 파이프라인
docs/                      수동 QA 및 프로젝트 문서
```

---

## 검증 가이드

Windows 실기 검증은 아래 체크리스트를 기준으로 진행합니다.

- `docs/windows-manual-qa-checklist.md`

기본 정적 검증:

```bash
npm run check
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc
```

---

## 배포 / 버전 관리

버전은 `CHANGELOG.md` 최신 항목을 기준으로 관리합니다.
GitHub Actions가 해당 버전을 읽어 beta / release 빌드에 주입합니다.

예시:
- `beta-vYY.MM.DD.####`
- `Release-vYY.MM.DD`

---

## 주의사항
- 이 도구는 시스템 보안/가상화 설정을 변경할 수 있습니다.
- 사용 전 테스트 환경 또는 스냅샷 환경을 권장합니다.
- 변경 사항 적용 후 재부팅이 필요할 수 있습니다.
