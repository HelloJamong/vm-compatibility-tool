# VM Compatibility Tool

Windows 환경에서 **VM 호환성 점검**과 **Hyper-V / WSL / VBS / 코어 격리 비활성화**를 지원하는
관리자 권한 기반 데스크톱 도구입니다.

## 📖 프로젝트 소개

VM Compatibility Tool은 Windows PC에서 가상 머신 호환성에 영향을 줄 수 있는 요소를 빠르게 점검하고,
필요 시 Hyper-V / WSL / VBS / 코어 격리 관련 설정을 선택적으로 정리할 수 있도록 만든 내부 운영 도구입니다.

현재 구현 기준으로 아래 흐름을 제공합니다.

- 앱 시작 시 시스템 정보 + 가상화 상태 자동 점검
- 점검 결과 CSV 자동 저장
- 시스템 정보 / 가상화 점검 패널 제공
- 선택형 비활성화 실행
- 추가 선택형 레지스트리 조치
- 로그 / 레지스트리 백업 / 재부팅 유도

## 🛠️ 기술 스택

### Backend
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-000000?style=for-the-badge&logo=rust&logoColor=white)
![Serde](https://img.shields.io/badge/Serde-D4AA00?style=for-the-badge&logo=rust&logoColor=black)

### Frontend
![Svelte](https://img.shields.io/badge/Svelte-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-06B6D4?style=for-the-badge&logo=tailwindcss&logoColor=white)

### Windows Integration
![WMI](https://img.shields.io/badge/WMI-0078D4?style=for-the-badge&logo=windows&logoColor=white)
![WinReg](https://img.shields.io/badge/Windows_Registry-0078D4?style=for-the-badge&logo=windows&logoColor=white)
![PowerShell](https://img.shields.io/badge/PowerShell-5391FE?style=for-the-badge&logo=powershell&logoColor=white)

### Build / Release
![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-2088FF?style=for-the-badge&logo=githubactions&logoColor=white)
![NSIS](https://img.shields.io/badge/NSIS-6D4AFF?style=for-the-badge)
![Windows](https://img.shields.io/badge/Windows_10/11-0078D4?style=for-the-badge&logo=windows&logoColor=white)

---

## ✨ 주요 기능

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
- WHfB(Windows Hello for Business) 경고 흐름
- 참고용 / 수동 선택형 legacy 레지스트리 항목 처리

### 3. 선택적 비활성화
가상화 점검 결과를 바탕으로 필요한 항목만 선택적으로 조치합니다.

- Hyper-V 비활성화
- WSL 비활성화
- VBS 관련 레지스트리 비활성화
- 코어 격리 관련 레지스트리 비활성화
- 추가 선택형 레지스트리 조치
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
  - `hypervisorlaunchtype off`
- 역할
  - Windows Hyper-V 하이퍼바이저와 관리 도구를 비활성화해 다른 VM / 에뮬레이터와의 충돌 가능성을 낮춥니다.
- 위험도: **중간**
  - Hyper-V 기반 VM, Android Emulator, Docker Desktop(설정에 따라), 일부 보안 기능이 동작하지 않을 수 있습니다.

#### 3-2. WSL 비활성화
- 제거 대상 기능
  - `Microsoft-Windows-Subsystem-Linux`
  - `VirtualMachinePlatform`
- 역할
  - WSL / WSL2 실행에 필요한 Windows 기능을 비활성화합니다.
- 위험도: **중간**
  - WSL 배포판 실행이 불가능해지고, WSL2 의존 개발 환경 / Docker 연계 구성이 영향을 받을 수 있습니다.

#### 3-3. VBS / 코어 격리 레지스트리 비활성화
- 원칙
  - 값이 실제로 존재하고 활성 상태일 때만 `0`으로 변경합니다.
  - 값이 없으면 새로 만들지 않습니다.
  - 조직 정책 / MDM / Windows Hello for Business 영향으로 재부팅 후 다시 활성화될 수 있습니다.
- 상세 문서
  - [VBS / 코어 격리 레지스트리 가이드](docs/vbs-registry-disable-guide.md)

### 4. 로그 / 백업 / 재부팅
- 점검 결과 CSV 자동 저장
- 운영 로그 저장
- 레지스트리 백업 저장
- 재부팅 예약 / 수동 재부팅 유도

---

## 🚀 빠른 시작

### 1. 저장소 준비

```bash
git clone https://github.com/HelloJamong/vm-compatibility-tool.git
cd vm-compatibility-tool
npm ci
```

### 2. 개발 실행

```bash
npm run tauri dev
```

### 3. 정적 점검

```bash
npm run check
```

### 4. 포터블 EXE 빌드

```bash
npm run tauri build -- --no-bundle
```

### 5. NSIS 인스톨러 빌드

```bash
npm run tauri build -- --bundles nsis
```

### 6. Windows용 대화형 빌드 스크립트

```bat
build.bat
```

---

## 🧩 시스템 요구사항

- Windows 10 / 11 64-bit
- 관리자 권한
- WebView2 사용 가능 환경

---

## 🔐 내부 배포 실행 안내

이 도구는 현재 **내부 배포용 실행 파일** 기준으로 사용됩니다.  
Windows 보안 정책 및 Microsoft SmartScreen 상태에 따라, 다운로드한 EXE 실행 시 경고 화면이 표시될 수 있습니다.

### SmartScreen 경고가 보일 때

1. EXE 실행
2. `Windows에서 PC를 보호했습니다` 화면이 나오면 **추가 정보** 클릭
3. 게시자 / 파일명을 다시 확인
4. 내부 배포 파일이 맞다면 **실행** 클릭

### 실행 전 확인 사항

- 배포 경로가 내부 공유 위치 또는 내부 공지된 릴리즈 링크인지 확인
- 파일명이 배포 공지와 일치하는지 확인
- 반드시 **관리자 권한으로 실행**

### 주의

- 이 경고는 현재 배포 방식상 나타날 수 있습니다.
- 일반 사용자용 공개 배포가 아니라 **내부 확인된 배포본**인지 먼저 확인한 뒤 진행하세요.

---

## 🏗️ 프로젝트 구조

```text
src/                       Svelte 앱 진입점 및 UI 컴포넌트
src/components/            UI 패널 / 공통 컴포넌트
src/lib/                   프론트 공용 타입
src-tauri/src/commands/    Tauri command 레이어
src-tauri/src/services/    Windows / WMI / Registry / Process 서비스
src-tauri/src/models/      Rust 직렬화 모델
src-tauri/src/main.rs      관리자 권한 확인 + 앱 진입점
src-tauri/build.rs         Windows manifest / Tauri build 설정
.github/workflows/         beta / release 빌드 파이프라인
docs/                      운영 / QA / 레지스트리 가이드 문서
```

**주요 디렉토리 설명**
- `src/components/`: 시작 점검 화면, 점검 패널, 비활성화 모달 등 UI 구성
- `src-tauri/src/commands/`: 프론트에서 호출하는 Tauri command 레이어
- `src-tauri/src/services/`: Windows 기능 / 프로세스 / 레지스트리 / WMI 접근 로직
- `.github/workflows/`: beta / release 빌드 및 배포 자동화
- `docs/`: 수동 QA 체크리스트 및 VBS 레지스트리 가이드

---

## 📚 추가 가이드

- [Windows 수동 QA 체크리스트](docs/windows-manual-qa-checklist.md)
- [VBS / 코어 격리 레지스트리 가이드](docs/vbs-registry-disable-guide.md)
- [변경 이력](CHANGELOG.md)

---

## ✅ 검증 가이드

기본 정적 검증:

```bash
npm run check
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc
```

Windows 실기 검증은 아래 문서를 기준으로 진행합니다.

- [Windows 수동 QA 체크리스트](docs/windows-manual-qa-checklist.md)

---

## 📦 배포 / 버전 관리

버전은 `CHANGELOG.md` 최신 항목을 기준으로 관리합니다.
GitHub Actions가 해당 버전을 읽어 beta / release 빌드에 주입합니다.

예시:
- `beta-vYY.MM.DD.####`
- `Release-vYY.MM.DD`

현재 배포 흐름:
- beta: 버전 포함 EXE 파일명 유지
- release: `vm-compatibility-tool.exe` 고정 파일명 사용

---

## ⚠️ 주의사항

- 이 도구는 시스템 보안 / 가상화 설정을 변경할 수 있습니다.
- 사용 전 테스트 환경 또는 스냅샷 환경을 권장합니다.
- 변경 사항 적용 후 재부팅이 필요할 수 있습니다.
- 조직 관리 PC에서는 로컬 조치보다 GPO / MDM 정책 확인이 우선일 수 있습니다.

---

## 📄 라이선스

이 프로젝트는 [MIT License](LICENSE)를 따릅니다.
