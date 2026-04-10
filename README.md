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
