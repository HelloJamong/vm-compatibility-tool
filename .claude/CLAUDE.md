# VM Compatibility Tool — 현재 프로젝트 컨텍스트

## 현재 상태 (2026-04-10 기준)

이 저장소의 현재 기준 구현은 **Tauri v2 + Rust + Svelte 5** 입니다.
기존 WPF/C# 코드는 마이그레이션 참조용으로 사용되었지만, 현재 저장소 런타임 기준은 Tauri 앱입니다.

### 현재 브랜치 운용
- 주 작업 브랜치: `beta`
- 현재 초점: **Stage 4 (시스템 정보 자동 저장) + Windows 실기 QA**

---

## 진행 상태 요약

| 영역 | 상태 | 메모 |
|------|------|------|
| Tauri 마이그레이션 | ✅ 완료 | 현재 앱 기준 구현 |
| 시스템 정보 수집 | ✅ 완료 | 이벤트 로그 포함 |
| 가상화 점검 | ✅ 완료 | Hyper-V / WSL / BCD / Registry |
| 선택적 비활성화 | ✅ 완료 | 점검 결과 기반 실행 |
| 레지스트리 manifest | ✅ 완료 | shared source of truth 도입 |
| GUI 구조 분리 | ✅ 완료 | `App.svelte` → 패널 컴포넌트 분리 |
| UI 간소화 (운영팀 최적화) | ✅ 완료 | 앱 시작 시 자동 점검, 조치 필요 항목만 표시 |
| WHfB 감지 + 경고 | ✅ 완료 | NGC/CloudDomainJoin/MDM 기반 감지, 경고 배너 표시 |
| 운영 로그 자동 저장 | ✅ 완료 | `{exe_dir}/logs/*.log` — 비활성화 실행 시 자동 기록 |
| 레지스트리 백업 (.reg) | ✅ 완료 | `{exe_dir}/logs/*_backup.reg` — UTF-16 LE BOM, 복원 가능 |
| 시스템 정보 CSV 자동 저장 | ⏳ Stage 4 | 수동 내보내기 제거 → 자동 저장으로 변경 |
| Windows 실기 QA | ⏳ 필요 | `docs/windows-manual-qa-checklist.md` 기준 |

---

## 현재 아키텍처

### Frontend
- `src/App.svelte` — 상태/오케스트레이션 셸
- `src/components/layout/` — 앱 헤더/상태바
- `src/components/menu/` — 메인 메뉴
- `src/components/system/` — 시스템 정보 패널
- `src/components/virtualization/` — 가상화 점검 패널
- `src/components/disable/` — 비활성화 패널
- `src/components/common/` — 공통 UI 컴포넌트 (`ConfirmDialog`, `StatusBadge`, `SummaryCard`)
- `src/lib/app-types.ts` — 프론트 공용 타입

### Backend
- `src-tauri/src/main.rs` — 관리자 권한 확인 + 진입점
- `src-tauri/src/lib.rs` — Tauri builder / command 등록
- `src-tauri/src/commands/`
  - `system_info.rs`
  - `virtualization.rs`
  - `disable.rs`
  - `export.rs`
- `src-tauri/src/services/`
  - `wmi_service.rs`
  - `registry_service.rs`
  - `registry_manifest.rs`
  - `process_service.rs`
  - `disk_service.rs`
  - `event_log_service.rs`
  - `log_service.rs`
- `src-tauri/src/models/`
  - `system_info.rs`
  - `virtualization.rs`

---

## 현재 구현 정책

### 레지스트리 정책
현재 프로젝트는 아래 정책으로 동작합니다.

1. **검사 시 값이 없으면 생성하지 않음**
   - `미설정`으로만 표시
2. **비활성화 시에도 없는 값은 생성하지 않음**
3. **실제로 존재하고 활성 상태인 값만 변경**
   - 목표값 `0`으로 조정
4. **legacy / reference-only 항목은 점검 결과에는 보이지만 자동 조치 대상 아님**

### manifest 정책
- 레지스트리 제어는 `registry_manifest.rs`를 source of truth로 사용
- 항목은 다음으로 분류됨
  - `DisableWrite`
  - `InspectOnly`
  - `ExcludedLegacy`

---

## 현재 주요 기능

### 시스템 정보
- OS / CPU / 메모리 / 디스크 / 부팅 / 메인보드 / GPU / 전원 계획
- 최근 이벤트 로그 요약
- CSV 내보내기

### 가상화 점검
- Hyper-V 상태
- WSL / VirtualMachinePlatform 상태
- `hypervisorlaunchtype`
- VBS / HVCI / Credential Guard / LSA 관련 레지스트리
- reference-only legacy 레지스트리 표시

### 가상화 점검 (추가)
- Windows Hello for Business(WHfB) 감지
  - NGC 폴더, CloudDomainJoin 레지스트리, PassportForWork 정책, MDM 등록 타입(6/13) 기반
  - `manifest_id: "whfb_check"` 로 구분 — 조치 대상 아님, 경고 전용
- UI: 조치 필요 항목만 테이블 표시 (전체 데이터는 CSV로만 제공)

### 비활성화
- Hyper-V 비활성화
- WSL 비활성화
- VBS 레지스트리 비활성화
- 코어 격리 레지스트리 비활성화
- 재부팅 요청
- **운영 로그 자동 저장**: `{exe_dir}\logs\YYYYMMDD_HHMMSS_{컴퓨터명}.log`
- **레지스트리 백업 자동 저장**: `{exe_dir}\logs\YYYYMMDD_HHMMSS_{컴퓨터명}_backup.reg`
  - UTF-16 LE BOM 형식 — Windows 레지스트리 편집기에서 직접 실행해 복원 가능

---

## 빌드 / 실행

### 개발
```bash
npm ci
npm run tauri dev
```

### 정적 점검
```bash
npm run check
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc
```

### 빌드
```bash
npm run tauri build -- --no-bundle
npm run tauri build -- --bundles nsis
```

### Windows 스크립트
- `build.bat` — 현재 Tauri 기준 빌드 스크립트

---

## QA 기준

Windows 실기 검증은 반드시 아래 문서를 기준으로 수행합니다.

- `docs/windows-manual-qa-checklist.md`

핵심 검증 포인트:
- 관리자 권한 실행
- 가상화 점검 결과 일관성
- missing-key 정책
- selective disable
- reboot/export 흐름
- 최소 창 크기(820x560) 가독성

---

## 남은 우선순위 작업

1. **Stage 4**: 시스템 정보 CSV 자동 저장 — 수동 내보내기 버튼 제거, 점검 완료 시 `{exe_dir}\logs\` 에 자동 저장
2. Windows 실기 QA 실행 (`docs/windows-manual-qa-checklist.md` 기준)
   - 로그/백업 파일 실제 생성 확인
   - WHfB 감지 경고 동작 확인
3. 검증 결과 기반 GUI 문구 / 시각 polish
4. manifest 세부 조정 필요 시 반영
5. 배포 전 changelog / release note 정리 (CHANGELOG 0006~0007)

---

## 레거시 코드에 대한 현재 입장

- 기존 WPF/C# 루트 파일은 현재 저장소 기준으로 제거됨
- 과거 구현 이력은 Git history에서 추적 가능
- 새 작업은 **Tauri/Rust/Svelte 기준으로만** 진행
- 레거시 동작 비교가 필요하면 Git history 또는 changelog 기반으로 확인

---

## 주의사항

- 이 프로젝트는 Windows 전용 성격이 강함
- 레지스트리/보안/가상화 설정 변경이 포함됨
- 가능하면 VM 또는 스냅샷 환경에서 먼저 검증
- Linux 워크스페이스에서는 컴파일/구조 검증까지만 가능하며, 최종 판단은 Windows 실기 QA 기준으로 해야 함
