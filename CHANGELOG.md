# Changelog

## [beta-v26.04.01.0001] - 2026-04-08

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
