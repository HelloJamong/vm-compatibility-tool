# Changelog

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
