# VM Compatibility Tool — 프로젝트 컨텍스트 및 개선 플랜

## 현재 진행 상태 (2026-04-07 기준)

| Phase | 상태 | 브랜치 |
|-------|------|--------|
| Phase 0 — 환경 구성 및 PoC | ✅ 완료 | nightly |
| Phase 1 — 데이터 모델 및 서비스 레이어 | ✅ 완료 | nightly |
| Phase 2 — Rust 커맨드 구현 | ✅ 완료 (이벤트 로그 미구현) | nightly |
| Phase 3 — Svelte 프론트엔드 | ✅ 완료 | nightly |
| Phase 4 — 통합 및 예외 처리 | ✅ 완료 | nightly |
| Phase 5 — 빌드 파이프라인 | ✅ 완료 | nightly |

**현재 브랜치:** `nightly` — main에 아직 머지하지 않음  
**다음 작업:** 실제 Windows 환경 검증 → nightly 빌드 테스트 → beta 배포

### 코드 서명 방침
- 내부 사용자 대상 이슈조치 도구이므로 **서명 없이 배포**
- SmartScreen 경고는 "추가 정보 → 실행"으로 통과 (내부 사용자 허용 범위)
- GPO 관리 환경이 생기는 경우 자체 서명 + 인증서 배포 방식으로 전환 가능
- `release.yml`에 `signtool.exe sign` 단계 자리는 주석으로 유지

### 구현된 파일 목록 (Tauri v2)

```
src-tauri/src/
├── main.rs               # 진입점 — 관리자 권한 체크 (TokenElevation + MessageBoxW)
├── commands/
│   ├── system_info.rs    # get_app_version, get_system_info
│   ├── virtualization.rs # get_virtualization_status
│   ├── disable.rs        # execute_disable, request_reboot (실패 시 log_service 연동)
│   └── export.rs         # export_csv
├── services/
│   ├── wmi_service.rs    # Win32_Processor/ComputerSystem/OS/VideoController/BaseBoard/DiskDrive/PowerPlan + MSFT_PhysicalDisk
│   ├── registry_service.rs # VBS/HVCI/CredGuard/LSA 읽기 + DWORD 쓰기 + OS 버전
│   ├── process_service.rs  # dism, bcdedit, shutdown 래퍼
│   ├── disk_service.rs     # SSD/HDD 타입 감지
│   └── log_service.rs      # 에러 로그 (%TEMP%\VMCompatibilityTool\error_YYYYMMDD.log)
└── models/
    ├── system_info.rs    # SystemInfoItem
    └── virtualization.rs # VirtualizationItem, DisableResult, ProgressEvent

src/
└── App.svelte            # 전체 UI (메뉴/시스템정보/가상화/비활성화 패널)

build.bat                 # Tauri 빌드 스크립트 (포터블 EXE / NSIS 선택)
```

### CHANGELOG.md 버전 작성 규칙

```
## [nightly-vYY.MM.DD.빌드번호] - YYYY-MM-DD   ← nightly (dev/nightly 브랜치 push)
## [beta-vYY.MM.DD.빌드번호] - YYYY-MM-DD      ← beta (beta 브랜치 push)
## [Release-vYY.MM.DD] - YYYY-MM-DD            ← release (Actions 수동 트리거)
```

---

## 프로젝트 개요

Windows OS 환경에서 VM(VMware, VirtualBox 등) 사용을 위해 시스템 호환성을 점검하고
Hyper-V, WSL, VBS 관련 설정을 비활성화하는 C# WPF 데스크탑 도구.

**현재 스택 → 목표 스택: C# WPF → Tauri v2 (Rust + Svelte)**

| 항목 | 현재 (C# WPF) | 목표 (Tauri v2) |
|------|--------------|----------------|
| 기술 스택 | C# 12, WPF, .NET 8.0-windows | Rust (백엔드) + Svelte (프론트엔드) |
| 빌드 타겟 | win-x64 단일 EXE | win-x64 단일 EXE (설치 또는 포터블) |
| 권한 | requireAdministrator (app.manifest) | requireAdministrator (tauri.conf.json + build.rs) |
| 런타임 의존성 | .NET 8 번들 (~80~150MB) | WebView2 (Win10/11 기본 내장) |
| 예상 EXE 크기 | ~80~150MB | ~5~10MB |
| 외부 패키지 | System.Management (WMI) | wmi, winreg, tokio, serde |

---

## 현재 구현 기능 정의 (C# WPF 기준)

### 기능 1: 시스템 사양 체크 (SystemInfo)

수집 항목 및 구현 방식:

| 수집 항목 | 데이터 소스 | 주요 메서드 |
|----------|-----------|-----------|
| OS 버전/빌드 | Registry `SOFTWARE\Microsoft\Windows NT\CurrentVersion` | `GetWindowsVersionInfo()` |
| CPU 모델/코어/클럭 | WMI `Win32_Processor` | `GetCpuInfo()` |
| 메모리 총용량/가용량 | WMI `Win32_ComputerSystem`, `Win32_PerfRawData_PerfOS_Memory` | `GetMemoryInfo()` |
| 디스크 용량/타입(SSD/HDD) | WMI `MSFT_PhysicalDisk`, `Win32_DiskDrive`, Registry | `GetDiskInfo()`, `GetDriveMediaType()` |
| 부팅 시간/가동 시간 | WMI `Win32_OperatingSystem`, EventLog, PerformanceCounter, Environment.TickCount64 | `GetBootTime()` |
| 메인보드 제조사/모델 | WMI `Win32_BaseBoard` | `CollectMotherboardInfoToTable()` |
| GPU 정보 | WMI `Win32_VideoController` | `CollectGraphicsCardInfoToTable()` |
| 전원 관리 설정 | WMI `Win32_PowerPlan`, Registry | `CollectPowerManagementInfoToTable()` |
| 이벤트 로그 요약 | `System.Diagnostics.EventLog` | `CollectEventLogInfoToTable()` |

출력: DataGrid 테이블 (Category / Item / Value 3열), CSV 내보내기 지원

---

### 기능 2: 가상화 설정 점검 (Virtualization)

점검 항목 및 구현 방식:

| 점검 항목 | 데이터 소스 | 판정 로직 |
|----------|-----------|---------|
| 하드웨어 가상화 지원 | WMI `Win32_Processor.VirtualizationFirmwareEnabled` | true/false |
| WSL 설치 상태 | `dism.exe /online /get-featureinfo` | State: Enabled/Disabled 파싱 |
| Hyper-V 설치 상태 (5개 구성요소) | `dism.exe /online /get-featureinfo` | 각 Feature별 상태 |
| Hypervisor 시작 유형 | `bcdedit.exe /enum {current}` | hypervisorlaunchtype 값 파싱 |
| VBS 활성화 여부 | Registry `SYSTEM\CurrentControlSet\Control\DeviceGuard` | `EnableVirtualizationBasedSecurity` 값 |
| HVCI 상태 | Registry `DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` | `Enabled` 값 |
| CredentialGuard 상태 | Registry `DeviceGuard\Scenarios\CredentialGuard` | `Enabled` 값 |
| LSA 보호 설정 | Registry `SYSTEM\CurrentControlSet\Control\Lsa` | `LsaCfgFlags` 값 |

출력: DataGrid 테이블 (Category / Status / Details / Recommendation 4열), CSV 내보내기 지원

점검 완료 후 결과를 메모리에 보관 → 비활성화 실행 시 필요한 항목만 선택적 조치

---

### 기능 3: VBS 및 Hyper-V 비활성화 (Disable)

실행 항목:

| 작업 | 방식 | 대상 |
|------|------|------|
| Hyper-V 기능 제거 | `dism.exe /online /disable-feature /norestart` | Microsoft-Hyper-V-All 외 6개 Feature |
| Hypervisor 시작 비활성화 | `bcdedit.exe /set hypervisorlaunchtype off` | BCD 부트 설정 |
| WSL 기능 제거 | `dism.exe /online /disable-feature /norestart` | Microsoft-Windows-Subsystem-Linux, VirtualMachinePlatform |
| VBS 레지스트리 비활성화 | `Registry.LocalMachine.CreateSubKey` + `SetValue(0)` | DeviceGuard 관련 30개 이상 항목 |
| 코어 격리 비활성화 | `Registry.LocalMachine.CreateSubKey` + `SetValue(0)` | DeviceGuard Policy 7개 항목 |

실행 모드:
- **선택적 모드**: 가상화 점검 결과를 기반으로 필요한 항목만 실행
- **전체 모드**: 점검 없이 실행 시 모든 항목 일괄 처리

완료 후: 재부팅 여부 확인 팝업 → 즉시 재부팅(shutdown /r /t 5) 또는 수동 재부팅 안내

---

### 기능 4: CSV 내보내기 (Export)

- 시스템 정보, 가상화 정보 각각 SaveFileDialog → CSV/TXT 선택
- UTF-8 BOM 인코딩 (한글 깨짐 방지)
- 헤더 행 (생성일시, 버전 정보 포함)

---

## Tauri v2 마이그레이션 플랜

### 기술 스택 결정

| 레이어 | 선택 | 이유 |
|--------|------|------|
| 백엔드 | Rust (Tauri v2) | Windows API 직접 접근, 메모리 안전, 작은 바이너리 |
| 프론트엔드 프레임워크 | Svelte 5 | 컴파일 타임 최적화, 런타임 없음, 번들 최소 |
| 빌드 도구 | Vite | 빠른 HMR, Svelte 공식 지원 |
| 스타일 | Tailwind CSS v4 | 유틸리티 클래스, 빌드 시 미사용 제거 |
| Rust 비동기 | tokio | Tauri 기본 런타임 |
| WMI | `wmi` crate | Win32 WMI COM 래퍼 |
| 레지스트리 | `winreg` crate | Windows 레지스트리 안전 접근 |
| 에러 처리 | `anyhow` crate | 체이닝 에러 컨텍스트 |
| 직렬화 | `serde` + `serde_json` | Tauri IPC 자동 직렬화 |

---

### 목표 프로젝트 구조

```
vm-compatibility-tool/
├── src-tauri/                          # Rust 백엔드
│   ├── src/
│   │   ├── main.rs                     # 진입점 (최소)
│   │   ├── lib.rs                      # Tauri 앱 빌더
│   │   ├── commands/                   # Tauri IPC 커맨드 (프론트엔드 호출 진입점)
│   │   │   ├── mod.rs
│   │   │   ├── system_info.rs          # get_system_info()
│   │   │   ├── virtualization.rs       # get_virtualization_status()
│   │   │   ├── disable.rs              # execute_disable()
│   │   │   └── export.rs              # export_csv()
│   │   ├── services/                   # 비즈니스 로직 (순수 Rust)
│   │   │   ├── mod.rs
│   │   │   ├── system_info_service.rs  # WMI 수집 오케스트레이터
│   │   │   ├── virtualization_service.rs
│   │   │   ├── disable_service.rs
│   │   │   ├── wmi_service.rs          # WMI 공통 래퍼
│   │   │   ├── registry_service.rs     # Registry 읽기/쓰기 래퍼
│   │   │   ├── process_service.rs      # dism.exe / bcdedit.exe 실행
│   │   │   └── disk_service.rs         # SSD/HDD 타입 감지
│   │   └── models/                     # 데이터 모델 (Serde 직렬화)
│   │       ├── mod.rs
│   │       ├── system_info.rs          # SystemInfoItem
│   │       └── virtualization.rs       # VirtualizationItem, DisableResult
│   ├── Cargo.toml
│   ├── tauri.conf.json                 # 앱 설정, 권한, 윈도우 크기
│   ├── capabilities/
│   │   └── default.json                # IPC 허용 커맨드 목록
│   └── build.rs                        # 빌드 스크립트 (manifest 등)
│
├── src/                                # Svelte 프론트엔드
│   ├── app.html
│   ├── main.ts
│   ├── App.svelte                      # 루트 컴포넌트 (패널 라우팅)
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Header.svelte
│   │   │   └── StatusBar.svelte
│   │   ├── panels/
│   │   │   ├── MenuPanel.svelte        # 메인 메뉴 (3개 버튼)
│   │   │   ├── SystemInfoPanel.svelte  # 시스템 정보 테이블
│   │   │   ├── VirtualizationPanel.svelte
│   │   │   └── DisablePanel.svelte
│   │   └── shared/
│   │       ├── DataTable.svelte        # 공통 테이블 컴포넌트
│   │       ├── ProgressLog.svelte      # 실시간 진행 로그
│   │       └── ConfirmDialog.svelte    # 확인 다이얼로그
│   ├── stores/
│   │   ├── systemInfo.ts              # Svelte store (시스템 정보 상태)
│   │   ├── virtualization.ts          # 가상화 점검 결과 + 완료 플래그
│   │   └── ui.ts                      # 현재 패널, 로딩 상태
│   └── lib/
│       ├── commands.ts                # invoke() 타입 래퍼 (타입 안전)
│       └── utils.ts                   # CSV 다운로드 등 유틸
│
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.js
└── build.bat                          # 기존 빌드 스크립트 Tauri 버전
```

---

### C# → Rust 기능 대응표

#### 데이터 수집 레이어

| C# 구현 | Rust 대응 | crate |
|---------|----------|-------|
| `ManagementObjectSearcher("SELECT * FROM Win32_Processor")` | `WMIConnection::query::<Win32_Processor>()` | `wmi` |
| `Registry.LocalMachine.OpenSubKey(path)` | `RegKey::open_subkey(path)` | `winreg` |
| `Registry.LocalMachine.CreateSubKey(path)` | `RegKey::create_subkey(path)` | `winreg` |
| `key.SetValue(name, value, RegistryValueKind.DWord)` | `regkey.set_value(name, &value_u32)` | `winreg` |
| `Process.Start("dism.exe", args)` | `Command::new("dism.exe").args(args).output()` | `std` |
| `ManagementDateTimeConverter.ToDateTime(str)` | 직접 파싱 (WMI datetime 포맷 → `chrono`) | `chrono` |
| `Environment.TickCount64` | `std::time::SystemTime` 또는 `winapi GetTickCount64` | `windows-sys` |

#### UI 레이어

| C# WPF | Svelte |
|--------|--------|
| `ObservableCollection<T>` + DataGrid 바인딩 | Svelte `$state` + `{#each}` 테이블 |
| `Dispatcher.BeginInvoke()` | Tauri 이벤트 `listen()` |
| `IProgress<T>` 진행 보고 | `app_handle.emit("progress", payload)` |
| `MessageBox.Show(YesNo)` | `ConfirmDialog.svelte` 커스텀 컴포넌트 |
| `SaveFileDialog` | Tauri `dialog::save()` |
| `StatusTextBlock.Text` | Svelte `$ui.status` store |

---

### 관리자 권한 처리 방식

Tauri v2에서 `requireAdministrator` 설정:

```toml
# Cargo.toml
[package.metadata.tauri]
# build.rs에서 윈도우 매니페스트 임베드
```

```rust
// build.rs
fn main() {
    // Windows 매니페스트 임베드 (관리자 권한 요청)
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest(r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
              <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
                <security>
                  <requestedPrivileges>
                    <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
                  </requestedPrivileges>
                </security>
              </trustInfo>
            </assembly>
        "#);
        res.compile().unwrap();
    }
    tauri_build::build()
}
```

---

### 핵심 Rust 코드 패턴 예시

#### WMI 수집 (wmi crate)

```rust
// services/wmi_service.rs
use wmi::{COMLibrary, WMIConnection};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename = "Win32_Processor")]
struct Win32Processor {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "NumberOfCores")]
    number_of_cores: u32,
    #[serde(rename = "NumberOfLogicalProcessors")]
    number_of_logical_processors: u32,
    #[serde(rename = "MaxClockSpeed")]
    max_clock_speed: u32,
    #[serde(rename = "VirtualizationFirmwareEnabled")]
    virtualization_firmware_enabled: bool,
}

pub fn get_cpu_info() -> anyhow::Result<Vec<SystemInfoItem>> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com.into())?;
    let processors: Vec<Win32Processor> = wmi.query()?;

    let mut items = vec![];
    if let Some(cpu) = processors.into_iter().next() {
        items.push(SystemInfoItem::new("프로세서", "모델", &cpu.name));
        items.push(SystemInfoItem::new("프로세서", "코어 수", &cpu.number_of_cores.to_string()));
        items.push(SystemInfoItem::new("프로세서", "논리 프로세서", &cpu.number_of_logical_processors.to_string()));
        items.push(SystemInfoItem::new("프로세서", "최대 클럭", &format!("{} MHz", cpu.max_clock_speed)));
    }
    Ok(items)
}
```

#### 레지스트리 읽기/쓰기

```rust
// services/registry_service.rs
use winreg::{RegKey, enums::*};

pub fn get_vbs_status() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\DeviceGuard")
        .and_then(|key| key.get_value::<u32, _>("EnableVirtualizationBasedSecurity"))
        .map(|v| v == 1)
        .unwrap_or(false)
}

pub fn set_registry_dword(path: &str, name: &str, value: u32) -> anyhow::Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm.create_subkey(path)?;
    key.set_value(name, &value)?;
    Ok(())
}
```

#### 프로세스 실행 (dism / bcdedit)

```rust
// services/process_service.rs
use std::process::Command;

pub struct ProcessResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

pub fn disable_windows_feature(feature_name: &str) -> ProcessResult {
    let output = Command::new("dism.exe")
        .args(["/online", "/disable-feature",
               &format!("/featurename:{}", feature_name),
               "/norestart"])
        .output();

    match output {
        Ok(out) => ProcessResult {
            success: out.status.success(),
            output: String::from_utf8_lossy(&out.stdout).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        },
        Err(e) => ProcessResult {
            success: false,
            output: e.to_string(),
            exit_code: -1,
        },
    }
}

pub fn set_hypervisor_launch_off() -> ProcessResult {
    let output = Command::new("bcdedit.exe")
        .args(["/set", "hypervisorlaunchtype", "off"])
        .output();
    // 동일 패턴...
}
```

#### Tauri 커맨드 + 실시간 진행 이벤트

```rust
// commands/disable.rs
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct ProgressEvent {
    step: u32,
    total: u32,
    message: String,
    success: bool,
}

#[tauri::command]
pub async fn execute_disable(
    app: AppHandle,
    selective: bool,
) -> Result<Vec<DisableResult>, String> {
    let steps = vec![
        ("Hyper-V 기능 비활성화", disable_hyperv_features),
        ("WSL 비활성화", disable_wsl),
        ("VBS 비활성화", disable_vbs),
        ("코어 격리 비활성화", disable_core_isolation),
    ];

    let total = steps.len() as u32;
    let mut results = vec![];

    for (i, (label, func)) in steps.iter().enumerate() {
        app.emit("disable-progress", ProgressEvent {
            step: i as u32 + 1,
            total,
            message: label.to_string(),
            success: true,
        }).ok();

        let result = func();
        results.push(result);
    }

    Ok(results)
}
```

#### Svelte 프론트엔드 진행 로그 수신

```typescript
// lib/commands.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export async function executeDisable(
  onProgress: (step: number, total: number, message: string) => void
) {
  const unlisten = await listen<ProgressEvent>('disable-progress', (event) => {
    onProgress(event.payload.step, event.payload.total, event.payload.message);
  });

  try {
    const results = await invoke<DisableResult[]>('execute_disable', { selective: true });
    return results;
  } finally {
    unlisten();
  }
}
```

---

### 마이그레이션 단계별 플랜

#### Phase 0 — 환경 구성 및 PoC ✅ 완료

- [x] Rust 툴체인 설치 (`rustup`, `cargo`)
- [x] Tauri CLI 설치 (`cargo install tauri-cli --version "^2"`)
- [x] Node.js 설치 + npm
- [x] `create-tauri-app` 으로 스캐폴딩 (Svelte + TypeScript 템플릿)
- [x] `wmi` crate PoC: Win32_Processor WMI 쿼리 성공 확인
- [x] `winreg` crate PoC: DeviceGuard 레지스트리 읽기/쓰기 확인
- [x] 관리자 권한 manifest 빌드 확인
- [x] nightly GitHub Actions 빌드 성공 (windows-latest 러너에서 EXE 생성 확인)

**검증 완료:** nightly EXE 실행 확인 (2026-04-07)

---

#### Phase 1 — 데이터 모델 및 서비스 레이어 ✅ 완료

- [x] `models/system_info.rs` — `SystemInfoItem` 구조체 (Serialize/Deserialize)
- [x] `models/virtualization.rs` — `VirtualizationItem`, `DisableResult`, `ProgressEvent` 구조체
- [x] `services/wmi_service.rs` — WMI 공통 연결 관리 (root\cimv2, Storage, Power 네임스페이스)
- [x] `services/registry_service.rs` — 읽기/쓰기/CreateSubKey 래퍼 + LSA 체크
- [x] `services/process_service.rs` — dism, bcdedit, shutdown 실행 래퍼
- [x] `services/disk_service.rs` — SSD/HDD 타입 감지 (MSFT_PhysicalDisk 우선, 모델명 키워드 폴백)

---

#### Phase 2 — Rust 커맨드 구현 ✅ 완료 (이벤트 로그 제외)

- [x] `commands/system_info.rs`
  - `get_app_version()` — `TAURI_DISPLAY_VERSION` 환경변수 반환
  - OS 정보 — Registry 기반 (25H2 빌드 번호 보정 포함)
  - CPU 정보 — WMI Win32_Processor (VT-x/AMD-V 상태 포함)
  - 메모리 — WMI Win32_ComputerSystem (총 용량) + Win32_OperatingSystem (가용 용량)
  - 디스크 — WMI Win32_DiskDrive + MSFT_PhysicalDisk (SSD/HDD 타입 판별)
  - 부팅 시간/가동 시간 — WMI Win32_OperatingSystem LastBootUpTime 파싱
  - 메인보드 — WMI Win32_BaseBoard
  - GPU/VRAM — WMI Win32_VideoController
  - 전원 계획 — WMI Win32_PowerPlan (root\cimv2\power)
  - [ ] `get_event_log_summary()` — 미구현 (windows-sys crate 추가 필요)
- [x] `commands/virtualization.rs`
  - 하드웨어 가상화 — WMI Win32_Processor.VirtualizationFirmwareEnabled
  - Hyper-V 상태 — DISM (2개 Feature)
  - WSL 상태 — DISM (2개 Feature)
  - Hypervisor 시작 유형 — bcdedit /enum {current}
  - VBS / HVCI / CredentialGuard — Registry
  - LSA 보호 — Registry LsaCfgFlags
- [x] `commands/disable.rs`
  - `execute_disable(selective: bool)` — 비활성화 실행 + `disable-progress` 이벤트
  - `request_reboot()` — shutdown /r /t 5
  - VBS 레지스트리 10개 항목 (CurrentControlSet + ControlSet001 양쪽)
  - 코어 격리 레지스트리 4개 항목
  - Hyper-V Features 7개 + bcdedit off
  - WSL Features 2개
- [x] `commands/export.rs` — UTF-8 BOM CSV 저장 (system / virtualization 두 타입)

---

#### Phase 3 — Svelte 프론트엔드 구현 ✅ 완료 (단일 파일 구조)

> 컴포넌트 분리 대신 `App.svelte` 단일 파일로 구현 (기능 동일)

- [x] 메인 메뉴 패널 (3개 버튼)
- [x] 시스템 정보 패널 — 테이블 + 새로고침 + CSV 내보내기
- [x] 가상화 점검 패널 — 테이블 + 재점검 + CSV 내보내기 + 상태 색상 표시
- [x] 비활성화 패널 — 경고 + 실행 + 실시간 progress 로그 + 재부팅 버튼
- [x] 하단 상태바 — 현재 상태 메시지 + 버전 표시
- [x] `disable-progress` Tauri 이벤트 실시간 수신 (listen)
- [x] CSV 저장 다이얼로그 (`@tauri-apps/plugin-dialog` save)
- [x] 버전 표시 — `get_app_version` invoke on mount

---

#### Phase 4 — 통합 및 예외 처리 (일부 완료)

- [x] 비활성화 실행 → 재부팅 다이얼로그 (`window.confirm` + `request_reboot`)
- [x] WMI 쿼리 실패 시 graceful fallback (항목별 "오류" 행 표시)
- [x] Windows 25H2 ControlSet001 레지스트리 경로 처리 유지
- [x] 버전 정보 자동 주입 (`TAURI_DISPLAY_VERSION` → `get_app_version` 커맨드)
- [x] 관리자 권한 없이 실행 시 안내 및 종료 처리 (build.rs manifest + 런타임 TokenElevation 체크)
- [x] 에러 로그 파일 저장 (`%TEMP%\VMCompatibilityTool\error_YYYYMMDD.log`)

---

#### Phase 5 — 빌드 파이프라인 (일부 완료)

- [x] `tauri build` 명령으로 단일 EXE 빌드 확인 (nightly 검증)
- [x] GitHub Actions 워크플로 3종 완성 (nightly / beta / release)
  - nightly: dev/nightly 브랜치 push → artifact 3일 보관
  - beta: beta 브랜치 push → GitHub Pre-release
  - release: 수동 트리거 → GitHub Release + 태그 중복 방지
- [x] `build.bat` Tauri 버전으로 재작성 (포터블 EXE / NSIS 선택)
- [x] `signtool.exe sign` — 내부 사용 도구이므로 서명 없이 배포 (SmartScreen 경고는 허용 범위)

---

### 주요 리스크 및 대응

| 리스크 | 심각도 | 대응 방안 |
|--------|--------|---------|
| `wmi` crate가 일부 WMI 클래스 미지원 | 중간 | `windows-rs` COM 직접 호출로 폴백 |
| WebView2 미설치 환경 (구형 LTSC) | 낮음 | `tauri.conf.json`에 WebView2 설치 없이 오류 안내 or NSIS 인스톨러로 WebView2 번들 |
| Rust 비동기 + Windows API 조합 복잡성 | 중간 | WMI 쿼리는 `tokio::task::spawn_blocking`으로 격리 |
| 관리자 권한 재실행 (UAC 프롬프트) 처리 | 낮음 | 매니페스트 requireAdministrator로 시작 시 UAC 처리, 별도 재실행 로직 불필요 |
| `MSFT_PhysicalDisk` WMI 네임스페이스 접근 | 낮음 | `root\Microsoft\Windows\Storage` 네임스페이스 명시 필요 (`wmi` crate 지원) |

---

### 작업 시 주의사항

- **Windows 전용**: 모든 Rust 코드는 `#[cfg(target_os = "windows")]` 또는 `windows-sys` 타겟. 크로스 컴파일 불필요.
- **WMI 초기화**: `COMLibrary::new()`는 스레드당 1회. `spawn_blocking` 내부에서 생성.
- **레지스트리 쓰기 테스트**: 반드시 VM 스냅샷 환경에서 진행. 레지스트리 복구 불가 항목 있음.
- **25H2 ControlSet001 대응**: `CurrentControlSet`과 `ControlSet001` 양쪽에 동일한 레지스트리 값 설정하는 로직은 그대로 유지.
- **실효 없는 Status 키 제거**: `KeyGuard\Status` 하위 런타임 복원 값들은 Rust 버전에서 처음부터 제외.
- **재부팅 처리**: `std::process::Command::new("shutdown").args(["/r", "/t", "5"])` 후 `std::process::exit(0)`.
- **Tauri 권한 시스템**: v2는 capability 기반. `capabilities/default.json`에 사용하는 모든 API 명시 필요.
- **빌드 명령**: `cargo tauri build` (포터블 EXE) 또는 `cargo tauri build --bundles nsis` (인스톨러).

---

## 구버전 C# 코드 참조 메모

Rust 구현 시 아래 C# 메서드를 1:1 참조:

| 구현 대상 | C# 원본 위치 |
|----------|------------|
| OS 버전 감지 (25H2 포함) | `GetWindowsVersionInfo()` L764 |
| SSD 타입 감지 4단계 폴백 | `GetDriveMediaType()` L1014 |
| VBS 레지스트리 전체 목록 | `DisableVBS()` L3199 (30개 항목) |
| 코어 격리 레지스트리 목록 | `DisableCoreIsolation()` L3290 (7개 항목) |
| Hyper-V Feature 목록 | `DisableHyperVFeatures()` L3040 (7개 Feature) |
| 가상화 점검 → 필요 여부 판정 | `CheckIfHyperVNeedsDisabling()` 외 4개 L2912 |
| 부팅 시간 수집 폴백 순서 | `GetBootTime()` L1563 (Environment 우선) |

---

## CI/CD 워크플로 설계

### 버전 체계

```
Release-v26.04.03          ← 정식 릴리즈
beta-v26.04.03.0001        ← 베타 (검증용)
nightly-v26.04.03.0001     ← 나이틀리 (테스트용)
         │  │  │  └── 빌드 번호 4자리 (GitHub run_number 자동 부여)
         │  │  └──── 월 내 순번 (수동 관리)
         │  └─────── 월 2자리
         └────────── 연도 끝 2자리
```

**버전 관리 규칙:**
- `CHANGELOG.md`가 **유일한 버전 소스** — 개발자가 수동 작성
- CI는 CHANGELOG.md 최신 항목을 읽어 버전 추출 및 빌드에 주입
- beta/nightly의 4자리 빌드 번호는 GitHub `run_number` 자동 부여
- Cargo.toml semver 변환: `v26.04.03` → `26.4.3` (선행 0 제거)

| 표시 버전 | Cargo.toml semver |
|----------|------------------|
| `Release-v26.04.03` | `26.4.3` |
| `beta-v26.04.03.0001` | `26.4.3-beta.1` |
| `nightly-v26.04.03.0001` | `26.4.3-nightly.1` |

---

### CHANGELOG.md 형식

```markdown
# Changelog

## [Release-v26.04.03] - 2026-04-07
### Added
- 시스템 정보 수집 기능 추가
### Fixed
- VBS 레지스트리 비활성화 오류 수정

---

## [beta-v26.04.03.0002] - 2026-04-06
### Fixed
- Windows 25H2 환경에서 Hyper-V 감지 오류 수정

---

## [beta-v26.04.03.0001] - 2026-04-05
### Added
- 초기 베타 빌드
```

---

### 파일 구조

```
.github/
├── workflows/
│   ├── nightly.yml          # dev 브랜치 push → artifact (3일 보관)
│   ├── beta.yml             # beta 브랜치 push → GitHub Pre-release
│   └── release.yml          # 수동 트리거 → 정식 GitHub Release
└── scripts/
    └── parse_version.py     # CHANGELOG.md 버전 파싱 공통 스크립트
```

---

### parse_version.py

```python
# .github/scripts/parse_version.py
import re, sys

def parse_changelog(path="CHANGELOG.md"):
    with open(path, encoding="utf-8") as f:
        content = f.read()

    pattern = r"## \[([^\]]+)\] - (\d{4}-\d{2}-\d{2})\n(.*?)(?=\n---|\n## \[|\Z)"
    sections = re.findall(pattern, content, re.DOTALL)
    if not sections:
        sys.exit("CHANGELOG.md에서 버전을 찾을 수 없습니다")

    version_str, date, body = sections[0]
    return {
        "display":  version_str,
        "semver":   to_semver(version_str),
        "date":     date,
        "body":     body.strip(),
        "type":     get_type(version_str),
    }

def to_semver(v):
    patterns = [
        (r"Release-v(\d+)\.(\d+)\.(\d+)$",
         lambda m: f"{int(m[1])}.{int(m[2])}.{int(m[3])}"),
        (r"beta-v(\d+)\.(\d+)\.(\d+)\.(\d+)$",
         lambda m: f"{int(m[1])}.{int(m[2])}.{int(m[3])}-beta.{int(m[4])}"),
        (r"nightly-v(\d+)\.(\d+)\.(\d+)\.(\d+)$",
         lambda m: f"{int(m[1])}.{int(m[2])}.{int(m[3])}-nightly.{int(m[4])}"),
    ]
    for pattern, fmt in patterns:
        m = re.match(pattern, v)
        if m:
            return fmt(m)
    sys.exit(f"알 수 없는 버전 형식: {v}")

def get_type(v):
    if v.startswith("Release-"):  return "release"
    if v.startswith("beta-"):     return "beta"
    if v.startswith("nightly-"):  return "nightly"
    return "unknown"

if __name__ == "__main__":
    info = parse_changelog()
    with open(sys.argv[1], "a", encoding="utf-8") as f:
        for k, v in info.items():
            if "\n" in str(v):
                f.write(f"{k}<<EOF\n{v}\nEOF\n")
            else:
                f.write(f"{k}={v}\n")
```

---

### nightly.yml

```yaml
name: Nightly Build
on:
  push:
    branches: [dev]
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Parse CHANGELOG.md
        id: ver
        run: python .github/scripts/parse_version.py $env:GITHUB_OUTPUT
        shell: pwsh

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Inject version into Cargo.toml
        shell: pwsh
        run: |
          (Get-Content src-tauri/Cargo.toml) `
            -replace 'version = ".*"', 'version = "${{ steps.ver.outputs.semver }}"' |
          Set-Content src-tauri/Cargo.toml

      - name: Build portable EXE
        run: npm run tauri build -- --no-bundle
        env:
          TAURI_DISPLAY_VERSION: ${{ steps.ver.outputs.display }}

      - name: Rename artifact
        shell: pwsh
        run: |
          Copy-Item "src-tauri/target/release/VM Compatibility Tool.exe" `
                    "VM-Compatibility-Tool-${{ steps.ver.outputs.display }}.exe"

      - uses: actions/upload-artifact@v4
        with:
          name: nightly-${{ steps.ver.outputs.display }}-${{ github.sha }}
          path: VM-Compatibility-Tool-*.exe
          retention-days: 3
```

---

### beta.yml

```yaml
name: Beta Build
on:
  push:
    branches: [beta]
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Parse CHANGELOG.md
        id: ver
        run: python .github/scripts/parse_version.py $env:GITHUB_OUTPUT
        shell: pwsh

      - name: Validate beta version
        shell: pwsh
        run: |
          if ("${{ steps.ver.outputs.type }}" -ne "beta") {
            Write-Error "CHANGELOG.md 최신 항목이 beta 형식이 아닙니다: ${{ steps.ver.outputs.display }}"
            exit 1
          }

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Inject version into Cargo.toml
        shell: pwsh
        run: |
          (Get-Content src-tauri/Cargo.toml) `
            -replace 'version = ".*"', 'version = "${{ steps.ver.outputs.semver }}"' |
          Set-Content src-tauri/Cargo.toml

      - name: Build portable EXE
        run: npm run tauri build -- --no-bundle
        env:
          TAURI_DISPLAY_VERSION: ${{ steps.ver.outputs.display }}

      - name: Rename artifact
        shell: pwsh
        run: |
          Copy-Item "src-tauri/target/release/VM Compatibility Tool.exe" `
                    "VM-Compatibility-Tool-${{ steps.ver.outputs.display }}.exe"

      - name: Create Beta Pre-release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ steps.ver.outputs.display }}
          name: "${{ steps.ver.outputs.display }}"
          prerelease: true
          files: VM-Compatibility-Tool-*.exe
          body: |
            ## 베타 빌드

            ${{ steps.ver.outputs.body }}

            ---
            > ⚠️ 이 빌드는 베타 버전입니다. 검증 목적으로만 사용하세요.
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

### release.yml

```yaml
name: Release Build
on:
  workflow_dispatch:   # 실수 방지를 위해 수동 트리거만 허용

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Parse CHANGELOG.md
        id: ver
        run: python .github/scripts/parse_version.py $env:GITHUB_OUTPUT
        shell: pwsh

      - name: Validate release version
        shell: pwsh
        run: |
          if ("${{ steps.ver.outputs.type }}" -ne "release") {
            Write-Error "CHANGELOG.md 최신 항목이 Release 형식이 아닙니다: ${{ steps.ver.outputs.display }}"
            exit 1
          }

      - name: Check tag does not exist
        shell: pwsh
        run: |
          git fetch --tags
          $tag = "${{ steps.ver.outputs.display }}"
          if (git tag -l $tag) {
            Write-Error "태그 $tag 가 이미 존재합니다. 중복 배포를 방지합니다."
            exit 1
          }

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Inject version into Cargo.toml
        shell: pwsh
        run: |
          (Get-Content src-tauri/Cargo.toml) `
            -replace 'version = ".*"', 'version = "${{ steps.ver.outputs.semver }}"' |
          Set-Content src-tauri/Cargo.toml

      - name: Build portable EXE
        run: npm run tauri build -- --no-bundle
        env:
          TAURI_DISPLAY_VERSION: ${{ steps.ver.outputs.display }}

      - name: Rename artifact
        shell: pwsh
        run: |
          Copy-Item "src-tauri/target/release/VM Compatibility Tool.exe" `
                    "VM-Compatibility-Tool-${{ steps.ver.outputs.display }}.exe"

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ steps.ver.outputs.display }}
          name: "${{ steps.ver.outputs.display }}"
          prerelease: false
          make_latest: true
          files: VM-Compatibility-Tool-*.exe
          body: |
            ## 변경 사항

            ${{ steps.ver.outputs.body }}

            ---
            📦 **사용 방법**: EXE 파일을 다운로드하여 관리자 권한으로 실행
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

### 운영 절차

```
[Nightly]
  1. dev 브랜치에 코드 push
  2. Actions 자동 실행 → artifact 생성 (3일 보관)
  3. Actions 탭에서 EXE 다운로드 후 테스트

[Beta]
  1. CHANGELOG.md 상단에 beta-v26.04.03.0001 섹션 작성
  2. beta 브랜치에 push
  3. Actions 자동 실행 → GitHub Pre-release 페이지 생성

[Release]
  1. CHANGELOG.md 상단에 Release-v26.04.03 섹션 작성 (변경 내용 포함)
  2. GitHub Actions 탭 → release.yml → Run workflow
  3. 정식 GitHub Release 생성 + EXE 첨부 + 태그 자동 생성

[공통 주의]
  - CHANGELOG.md에 해당 버전 항목 없으면 빌드 실패 (의도된 안전장치)
  - 이미 존재하는 태그로 Release 시도 시 빌드 실패 (중복 배포 방지)
  - Release 워크플로는 수동 트리거만 허용 (실수 방지)
```
