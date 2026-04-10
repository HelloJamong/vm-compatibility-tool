# Changelog

## [beta-v26.04.01.0014] - 2026-04-10

### Changed
- 시작 점검 UI를 모달 오버레이 방식에서 팝업형 단독 화면 구조로 전환
  - 어두운 배경 오버레이 제거
  - 화면 중앙의 독립형 시작 패널로 동작하도록 조정
- 로컬 비교 이미지(`3.PNG`, `4.PNG`)는 다시 일반 파일로 취급하도록 ignore 해제

---

## [beta-v26.04.01.0013] - 2026-04-10

### Changed
- `design_sample.html` 재작성본을 기준으로 자동 점검 팝업 UI를 다시 정렬
  - 팝업 폭/비율을 샘플 기준으로 축소
  - 아이콘 / 타이포 / 여백 / summary card / 버튼 스타일 재조정
  - 전체 톤을 더 정적인 내부 도구용 팝업 스타일로 정리

### Fixed
- 로컬 디자인 참고 파일 ignore 정책 보강
  - `design_sample.html`
  - `3.PNG`
  - `4.PNG`

---

## [beta-v26.04.01.0012] - 2026-04-10

### Fixed
- 팝업 GUI 스타일이 실제 앱에서 적용되지 않던 문제 수정
  - `src/main.ts`에서 전역 스타일 로드 추가
  - `src/app.css` 추가
- 자동 점검 팝업을 목표 시안에 맞게 시각적으로 재정렬
  - 큰 제목/설명문 간격 조정
  - 요약 박스 / 버튼 / 하단 footer strip 스타일 개선
  - 점검 중 / 점검 완료 상태 표현 정리

### Changed
- 비교용 로컬 이미지(`3.PNG`, `4.PNG`)를 참고 파일로만 사용하도록 ignore 처리

---

## [beta-v26.04.01.0011] - 2026-04-10

### Removed
- 더 이상 사용하지 않는 nightly GitHub Actions 워크플로 제거
  - `.github/workflows/nightly.yml`

### Changed
- README 배포 설명을 현재 운영 기준인 beta / release 파이프라인 중심으로 정리
- nightly 브랜치는 히스토리 보존 용도로 남기고, 자동 빌드 대상에서는 제외

---

## [beta-v26.04.01.0010] - 2026-04-10

### Added
- 자동 점검 결과 팝업 UI 추가
  - 점검 중 / 점검 완료 상태 표시
  - 진행률 바 + % 표시
  - 조치 필요 항목 요약 표시
  - `조치 시작` / `닫기` 버튼 분기 처리
- 팝업 요약 문구 출력을 위한 자동 점검 결과 CSV 저장 경로 추가
  - `export_csv_auto` 커맨드
  - `{exe_dir}/logs/` 하위 자동 저장

### Changed
- 앱 시작 자동 점검 완료 후 상태 메시지를 저장 파일명 기준으로 보강
- 디자인 샘플 HTML(`design_sample.html`)을 로컬 참고 파일로만 사용하도록 ignore 처리

---

## [beta-v26.04.01.0009] - 2026-04-10

### Changed
- 내부 배포 실행 안내 보강
  - `README.md`에 SmartScreen 경고 대응 절차 추가
  - beta / release 릴리즈 노트에 `추가 정보 → 실행` 안내 추가

### Docs
- Windows 수동 QA 체크리스트에 SmartScreen 노출/안내 문구 일치 여부 점검 항목 추가

---

## [beta-v26.04.01.0008] - 2026-04-10

### Added
- beta / release GitHub Actions에 내부용 코드 서명 단계 추가
  - `WINDOWS_CODESIGN_PFX_BASE64`
  - `WINDOWS_CODESIGN_PFX_PASSWORD`
  - `WINDOWS_CODESIGN_TIMESTAMP_URL`(선택)
- 공용 PowerShell 서명 스크립트 추가
  - `.github/scripts/sign_windows_binary.ps1`

### Changed
- 내부 배포 환경을 고려해 self-signed / 사설 CA PFX 인증서로도 릴리즈 EXE를 서명할 수 있게 워크플로를 구성
- 서명 secret 이 없을 때는 빌드를 깨지 않고 summary에 skip 사유를 남기도록 조정

### Note
- 자체 서명만으로는 일반 외부 PC의 SmartScreen 경고가 자동으로 사라지지 않을 수 있으며, 내부 배포 대상 PC에 루트/중간 인증서 신뢰 배포가 필요함

---

## [beta-v26.04.01.0007] - 2026-04-10

### Added
- 작업 기준 문서 통합
  - `docs/unified-project-plan.md` 추가
  - 기능 정의 / 현재 상태 / TODO / 우선순위를 단일 문서 기준으로 정리
- Stage 4 설계 문서 추가
  - `docs/stage4-system-info-auto-save-plan.md`
  - 시스템 정보 CSV 자동 저장 구현 전 검토용 설계안 작성

### Changed
- `.claude/CLAUDE.md`를 중복 상태 문서에서 Claude/Codex 공용 포인터 문서로 단순화
- 정리 단계에 맞춰 저장소 기준 문서 역할을 `docs/unified-project-plan.md` 중심으로 재정렬

### Fixed
- Rust 모델에 `DisableOutput` 정의를 반영해 현재 비활성화 결과 구조와 프론트 타입 계약을 일치시킴
- `App.svelte`의 미사용 helper 제거로 사전 정리 상태를 정돈
- `.claude/settings.local.json`을 ignore에 추가해 로컬 전용 설정 파일이 추적되지 않도록 보완

---

## [beta-v26.04.01.0006] - 2026-04-10

### Fixed
- 가상화 점검 결과 배지 tone 오류 수정
  - 상태 문자열 매칭 방식에서 `action_required` 필드 기반으로 변경
  - `확인 불가`, `미설정` 항목이 잘못 녹색(success)으로 표시되던 문제 수정
  - `확인 불가` 행 hover 색상도 녹색 → 회색으로 수정
- 비활성화 패널 작업 목록 문구 수정
  - `WSL2 제거 (DISM)` → `WSL 기능 제거 (DISM)` (WSL1 / WSL2 모두 대상)
  - `hypervisorlaunchtype off (bcdedit)` → `Hypervisor 시작 유형 비활성화 (bcdedit)` (다른 항목과 형식 통일)

### Docs
- 최소 창 크기 기준 `900x600` → `820x560` 으로 문서 통일
  - `.claude/CLAUDE.md` QA 기준 항목 수정
  - `docs/windows-manual-qa-checklist.md` Regression Checks 항목 수정

---

## [beta-v26.04.01.0005] - 2026-04-08

### Changed
- 1차 UI 밀도 개선
  - 초기 메뉴를 더 컴팩트한 3버튼 중심 레이아웃으로 조정
  - 과도한 여백을 줄이고 패널 간격을 축소
  - 기본 창 크기 `980x640`, 최소 크기 `820x560`으로 조정
- 하단 상태 영역 재구성
  - 버전 정보를 좌측 하단에 고정
  - 상태 메시지와 진행 프로그레스바를 우측 하단에 배치
- 시스템 정보 / 가상화 점검 표 밀도 개선
  - 행/열 정렬을 더 촘촘하게 조정
  - 열 폭 및 셀 래핑을 개선해 가독성 향상

### Fixed
- 데이터 수집 중 진행 상태가 화면 중앙 로딩 표시만 보이던 문제를 전역 하단 진행 바로 보완
- 리스트 표에서 행/열 배치가 어색하게 보이던 구성을 1차 정리

---

## [beta-v26.04.01.0004] - 2026-04-08

### Added
- 공용 레지스트리 manifest 도입 (`registry_manifest.rs`)
  - VBS / 코어 격리 레지스트리 항목을 `DisableWrite` / `InspectOnly` / `ExcludedLegacy`로 분류
  - 점검/조치가 같은 source of truth를 공유하도록 정리
- Windows 수동 QA 체크리스트 추가 (`docs/windows-manual-qa-checklist.md`)
- GUI 공통 컴포넌트 추가
  - `StatusBadge`, `SummaryCard`, `ConfirmDialog`
  - 메뉴/가상화 점검/비활성화 패널 컴포넌트 분리

### Changed
- 레지스트리 비활성화 정책 정리
  - 값이 없는 레지스트리는 생성하지 않음
  - 실제로 존재하고 활성 상태인 값만 `0`으로 변경
- 가상화 점검 결과에 structured metadata 추가
  - `disable_group`, `action_required`, `manifest_id`, `source_type`
- `App.svelte`를 오케스트레이션 셸로 단순화하고 UI를 컴포넌트 중심 구조로 재편
- README 및 `.claude/CLAUDE.md`를 현재 Tauri/Rust/Svelte 기준으로 재작성

### Fixed
- selective 비활성화가 표시 문자열 매칭에 의존하던 구조 제거
- reference-only legacy 레지스트리 항목을 자동 조치 없이 점검 결과에 다시 노출
- 재부팅 흐름을 브라우저 `window.confirm` 대신 인앱 확인 다이얼로그로 개선
- 저장소 루트의 obsolete WPF/C# 유산 및 과도한 Visual Studio 중심 `.gitignore` 정리

---

## [beta-v26.04.01.0003] - 2026-04-08

### Added
- `DisableOptions` 구조체 도입 — selective 비활성화 로직 구현
  - 가상화 점검 완료 시 결과 기반으로 필요한 항목만 선택 실행
  - Hyper-V / WSL / VBS / 코어 격리 각각 독립적으로 on/off 제어
  - 점검 없이 실행 시 전체 항목 일괄 실행 (기존 동작 유지)
- 이벤트 로그 수집 구현 (`event_log_service`)
  - 최근 7일간 System / Application 로그 Level별 건수 (위험/오류/경고)
  - 최근 오류·위험 이벤트 5건 (시간/로그명/ID/메시지 미리보기)
  - PowerShell Get-WinEvent 기반, 관리자 권한 환경에서 동작
- 비활성화 진행 이벤트 개선
  - 태스크 실패 시 `disable-progress` 이벤트 재전송 (`success: false`)
  - 실행 대상 없을 때 "비활성화 필요 항목 없음" 결과 반환

### Fixed
- `execute_disable` 커맨드 시그니처 변경: `selective: bool` → `options: Option<DisableOptions>`
- Beta/Nightly/Release 포터블 빌드 오류 수정: `--bundles none` → `--no-bundle`
- EXE 아이콘 검은 네모 문제: `src-tauri/icons/icon.ico`를 멀티사이즈 ICO(16/24/32/48/64/128/256)로 재생성
- 시스템 정보의 최근 오류/위험 메시지 한글 깨짐 수정: PowerShell 출력 인코딩을 UTF-8로 강제
- 실행 시 "TaskDialogIndirect를 찾을 수 없습니다" 오류: manifest에 `Microsoft.Windows.Common-Controls v6` 의존성 추가

---

## [nightly-v26.04.01.0001] - 2026-04-07

### Added
- Tauri v2 (Rust + Svelte) 기반으로 프로젝트 마이그레이션 시작
- Phase 0 PoC: 프로젝트 기본 구조 스캐폴딩
  - Rust 백엔드 서비스 레이어 (wmi_service, registry_service, process_service, disk_service)
  - Tauri 커맨드 구조 (system_info, virtualization, disable, export)
  - Svelte 5 프론트엔드 기본 UI
- GitHub Actions nightly 빌드 워크플로 추가
- CHANGELOG.md 기반 버전 관리 체계 도입

### Note
- Phase 0: 구조 검증용 빌드 (WMI/Registry 실제 동작은 Phase 1에서 완성)
