# Stage 4 설계안 — 시스템 정보 CSV 자동 저장

> Last Updated: 2026-04-10  
> Base Document: `docs/unified-project-plan.md`  
> Status: Draft for review before implementation

---

## 1. 목적

Stage 4의 목표는 **시스템 정보 수집 결과를 사용자가 수동으로 내보내지 않아도 자동으로 CSV 파일로 저장**하도록 바꾸는 것이다.

이 설계안은 구현 전에 다음을 명확히 하기 위해 작성한다.

- 현재 구조에서 어디를 바꿔야 하는지
- 어떤 방식이 가장 작은 리스크로 목표를 달성하는지
- 완료 조건과 검증 범위가 무엇인지

이 문서는 현재 단일 기준 문서인 `docs/unified-project-plan.md`의 Stage 4 항목을 구체 구현 계획으로 확장한 문서다 (`docs/unified-project-plan.md:183-208`).

---

## 2. 요구사항 요약

### 제품 요구사항
`docs/unified-project-plan.md` 기준 Stage 4 요구사항은 아래와 같다.

- 시스템 정보 수집 완료 시 `{exe_dir}/logs/` 하위에 자동 저장 (`docs/unified-project-plan.md:195-199`)
- 예시 파일명: `YYYYMMDD_HHMMSS_{COMPUTERNAME}_system_info.csv` (`docs/unified-project-plan.md:197-198`)
- Excel / 메모장에서 한글 깨짐 없이 열려야 함 (`docs/unified-project-plan.md:205-208`)
- 저장 위치를 상태 메시지 또는 UI에서 확인할 수 있어야 함 (`docs/unified-project-plan.md:205-208`)
- 현재 수동 CSV 버튼은 제거 방향이 우선이다 (`docs/unified-project-plan.md:199-203`)

### QA 요구사항
현재 수동 export 기준 QA 항목은 아래와 같다.

- 시스템 정보 로드 성공 (`docs/windows-manual-qa-checklist.md:25-29`)
- CSV 결과가 Excel / Notepad에서 정상 열림 (`docs/windows-manual-qa-checklist.md:25-29`)

Stage 4 구현 후에는 이 항목을 **자동 저장 기준으로 재정의**해야 한다.

---

## 3. 현재 구현 상태

### 프론트엔드 현재 흐름
- 앱 시작 시 `fetchSystemInfo()`가 자동 호출된다 (`src/App.svelte:41-50`).
- 시스템 정보는 `invoke("get_system_info")`로 받아서 `systemItems`에 저장한다 (`src/App.svelte:57-69`).
- 별도 저장은 `exportSystemCsv()`에서만 수행되며, 사용자가 직접 경로를 선택해야 한다 (`src/App.svelte:109-126`).
- 시스템 정보 패널도 현재는 `CSV 내보내기` 버튼을 전제로 구성되어 있다 (`src/components/system/SystemInfoPanel.svelte:36-49`).

### 백엔드 현재 흐름
- `get_system_info`는 현재 `Vec<SystemInfoItem>`만 반환한다 (`src-tauri/src/commands/system_info.rs:16-24`).
- CSV 생성 로직은 `export_csv` 커맨드에 있고, 프론트가 넘긴 경로에 파일을 쓴다 (`src-tauri/src/commands/export.rs:6-19`, `src-tauri/src/commands/export.rs:21-69`).
- CSV는 이미 UTF-8 BOM으로 저장하고 있어 한글 깨짐 대응 기반은 존재한다 (`src-tauri/src/commands/export.rs:27-34`, `src-tauri/src/commands/export.rs:67-69`).
- 비활성화 로그/백업은 이미 `{exe_dir}/logs` 기준 자동 저장 공용 경로를 사용하고 있다 (`src-tauri/src/services/log_service.rs:14-18`, `src-tauri/src/services/log_service.rs:66-86`).

### 현재 구조의 핵심 제약
- 프론트엔드는 현재 `exe_dir/logs` 경로를 직접 알지 못한다.
- 따라서 Stage 4 자동 저장은 **백엔드 주도 저장**으로 구현하는 것이 자연스럽다.
- 현재 `get_system_info`는 저장 결과(path)를 함께 반환하지 않기 때문에, Stage 4에서는 응답 모델 변경 또는 후속 저장 커맨드 추가가 필요하다.

---

## 4. 설계 목표

이번 Stage 4에서 달성해야 할 설계 목표는 아래 5가지다.

1. **자동 저장을 기본 동작으로 만든다.**
2. **저장 경로는 기존 disable 로그/백업과 같은 `{exe_dir}/logs` 체계를 재사용한다.**
3. **프론트는 저장 경로를 표시할 수 있어야 한다.**
4. **시스템 정보 패널은 수동 export 중심 UI에서 자동 저장 완료 중심 UI로 바뀌어야 한다.**
5. **가상화 CSV export는 기존처럼 수동 유지한다.**

---

## 5. 설계 옵션

## 옵션 A — 프론트에서 2단계 호출

### 방식
1. `get_system_info`로 시스템 정보 수집
2. 이어서 `save_system_info_csv_auto(items)` 같은 별도 커맨드 호출

### 장점
- 기존 `get_system_info` 반환 타입을 바꾸지 않아도 됨
- 단계별 구현이 단순해 보임

### 단점
- 수집 성공 후 저장 실패가 별도 단계로 분리되어 흐름이 분산됨
- 프론트 코드가 저장 책임까지 추가로 갖게 됨
- 자동 저장이 “수집의 일부”가 아니라 “후처리”처럼 되어 계약이 약함
- 이후 다른 UI 경로에서 시스템 정보를 재사용할 때 저장 호출 누락 가능성 존재

### 판단
비추천. Stage 4는 제품 정책상 “자동 저장이 기본 동작”이므로, 수집과 저장이 분리된 흐름은 장기적으로 누락 위험이 있다.

---

## 옵션 B — 백엔드에서 수집 + 자동 저장을 하나의 응답으로 반환

### 방식
- `get_system_info`가 더 이상 `Vec<SystemInfoItem>`만 반환하지 않고,
  `items + auto_saved_csv_path`를 포함한 구조체를 반환한다.

예상 응답 형태:
- `items: SystemInfoItem[]`
- `auto_saved_csv_path: string | null`

### 장점
- 수집과 자동 저장이 하나의 계약으로 묶임
- 프론트는 결과 표시만 담당하면 됨
- 앱 시작 자동 수집과 수동 재수집 모두 동일한 저장 규칙 적용 가능
- 저장 실패를 UI에 자연스럽게 반영 가능

### 단점
- Rust 모델 / TypeScript 타입 / `App.svelte` 호출부를 함께 수정해야 함
- 현재 `get_system_info`를 쓰는 경로가 바뀌므로 영향 범위가 A보다 큼

### 판단
**권장안.** Stage 4 요구사항과 현재 구조를 가장 자연스럽게 연결한다.

---

## 옵션 C — 기존 `export_csv`를 확장해 자동 경로 모드 추가

### 방식
- `export_csv`에 `file_path` 없이도 동작하는 자동 모드를 추가하고,
- 프론트는 시스템 정보 수집 후 기존 export 커맨드를 재호출

### 장점
- CSV 생성 로직 재사용 가능

### 단점
- `export_csv`의 의미가 “사용자 export”와 “자동 저장”을 동시에 떠안게 됨
- 인터페이스가 모호해짐
- 결국 프론트의 2단계 호출 문제가 다시 생김

### 판단
비추천. 재사용성은 있지만 계약이 불분명해진다.

---

## 6. 권장 설계(ADR)

### Decision
**옵션 B 채택:** `get_system_info`가 시스템 정보 수집 결과와 자동 저장 경로를 함께 반환하도록 바꾼다.

### Drivers
- 자동 저장을 제품 기본 동작으로 보장해야 함
- `{exe_dir}/logs` 저장 규칙을 백엔드에서 일관되게 관리해야 함
- 프론트엔드는 저장 위치 표시와 상태 반영에 집중해야 함
- 앱 시작 자동 수집과 수동 재수집에서 같은 계약을 유지해야 함

### Alternatives considered
- 옵션 A: 프론트 2단계 호출
- 옵션 C: 기존 export 커맨드 자동 모드 확장

### Why chosen
- 누락 위험이 가장 낮고
- 제품 정책과 코드 계약이 가장 잘 맞으며
- 시스템 정보 수집이라는 단일 사용자 액션에 “자동 저장 결과”를 자연스럽게 포함시킬 수 있기 때문이다.

### Consequences
- `get_system_info` 반환 타입 변경이 필요하다.
- TypeScript 타입과 `App.svelte` 호출부를 같이 바꿔야 한다.
- `SystemInfoPanel` UX도 “수동 export”에서 “자동 저장 확인” 중심으로 바뀐다.

### Follow-ups
- Windows QA 체크리스트의 System Info 항목을 Stage 4 기준으로 갱신
- CHANGELOG에 자동 저장 전환 내역 반영

---

## 7. 변경 범위

### 백엔드
1. `src-tauri/src/models/system_info.rs`
   - `SystemInfoItem` 외에 `SystemInfoOutput` 추가
2. `src-tauri/src/commands/system_info.rs`
   - `get_system_info`가 자동 저장까지 수행하고 path 포함 응답 반환
3. `src-tauri/src/commands/export.rs`
   - CSV 생성 로직 재사용 방식 정리
4. `src-tauri/src/services/log_service.rs`
   - `{exe_dir}/logs` 경로 재사용 helper를 외부에서도 사용할 수 있게 정리
5. 필요 시 `src-tauri/src/services/` 하위에 CSV 저장 helper 분리

### 프론트엔드
1. `src/lib/app-types.ts`
   - `SystemInfoOutput` 타입 추가
2. `src/App.svelte`
   - `fetchSystemInfo()` 응답 처리 변경
   - 상태 메시지에 자동 저장 경로 반영
   - `exportSystemCsv()` 제거
3. `src/components/system/SystemInfoPanel.svelte`
   - `CSV 내보내기` 버튼 제거
   - 자동 저장 완료 메시지 / 저장 경로 / 재수집 안내 UI로 전환

### 문서
1. `docs/unified-project-plan.md`
   - Stage 4 상태 반영
2. `docs/windows-manual-qa-checklist.md`
   - 수동 export QA → 자동 저장 QA로 수정
3. `CHANGELOG.md`
   - Stage 4 반영 시 버전 항목 추가

---

## 8. 세부 구현 플랜

## Step 1 — 응답 모델 재정의

### 작업
- Rust에 `SystemInfoOutput` 구조체를 추가한다.
- TypeScript에 동일한 응답 타입을 추가한다.

### 대상 파일
- `src-tauri/src/models/system_info.rs`
- `src/lib/app-types.ts`

### 기대 결과
- 시스템 정보 커맨드는 `items`와 `auto_saved_csv_path`를 함께 반환할 수 있다.

---

## Step 2 — CSV 저장 책임을 백엔드로 이동

### 작업
- 시스템 정보 수집 직후 자동 저장 helper 호출
- 파일명 규칙: `YYYYMMDD_HHMMSS_{COMPUTERNAME}_system_info.csv`
- 저장 위치: `{exe_dir}/logs/`
- 기존 CSV BOM 로직 재사용

### 대상 파일
- `src-tauri/src/commands/system_info.rs`
- `src-tauri/src/commands/export.rs`
- `src-tauri/src/services/log_service.rs`

### 구현 메모
권장 구현 순서:
1. CSV 문자열 생성 로직을 재사용 가능한 함수로 분리
2. `log_service.rs`에서 logs 디렉토리 경로 helper를 외부 사용 가능하게 노출
3. `get_system_info` 내부에서 수집 → 저장 → 경로 반환 순서로 구성

### 기대 결과
- 프론트가 별도 저장 명령을 내리지 않아도 자동 저장이 완료된다.

---

## Step 3 — 프론트 상태/문구 전환

### 작업
- `fetchSystemInfo()`가 새로운 응답 구조를 받도록 변경
- 상태 메시지를 `수집 완료`에서 `수집 완료 — 자동 저장됨` 형태로 강화
- 자동 저장 실패 시에는 수집은 성공했지만 저장은 실패했다는 것을 분리 표시

### 대상 파일
- `src/App.svelte`

### 기대 결과
- 사용자는 저장 여부와 저장 위치를 즉시 알 수 있다.

---

## Step 4 — System Info 패널 UX 재구성

### 작업
- `CSV 내보내기` 버튼 제거
- 카드 문구를 자동 저장 중심으로 변경
- 저장 경로 또는 저장 완료 안내를 카드 내부에 표시
- `재수집` 버튼은 유지

### 대상 파일
- `src/components/system/SystemInfoPanel.svelte`

### 기대 결과
- UI가 제품 정책과 일치한다.
- 사용자는 별도 액션 없이 저장이 끝났음을 이해할 수 있다.

---

## Step 5 — 문서 및 QA 기준 정리

### 작업
- 통합문서에 Stage 4 완료 상태 반영
- QA 체크리스트를 자동 저장 기준으로 수정
- CHANGELOG에 Stage 4 반영

### 대상 파일
- `docs/unified-project-plan.md`
- `docs/windows-manual-qa-checklist.md`
- `CHANGELOG.md`

### 기대 결과
- 코드/문서/QA 기준이 다시 분산되지 않는다.

---

## 9. 수용 기준 (Acceptance Criteria)

1. 앱 시작 자동 수집 시 시스템 정보 CSV가 `{exe_dir}/logs/`에 자동 생성된다.
2. `재수집` 실행 시 새 CSV가 다시 생성된다.
3. 파일명은 `YYYYMMDD_HHMMSS_{COMPUTERNAME}_system_info.csv` 규칙을 따른다.
4. CSV는 Excel / 메모장에서 한글 깨짐 없이 열린다.
5. 시스템 정보 패널에 더 이상 수동 `CSV 내보내기` 버튼이 없다.
6. 시스템 정보 패널 또는 상태바에서 자동 저장 완료 및 저장 위치를 확인할 수 있다.
7. 가상화 CSV export 동작은 유지된다.
8. `npm run check` 통과
9. `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc` 통과

---

## 10. 리스크와 대응

### 리스크 1 — 앱 시작 시 파일이 너무 자주 생성될 수 있음
설명:
- 현재 앱은 시작 시 자동 수집을 수행한다 (`src/App.svelte:41-50`).
- 따라서 앱 실행만으로 CSV가 1회 생성된다.

대응:
- 이 동작은 Stage 4 요구사항과 일치하므로 기본 정책으로 수용한다.
- 다만 QA에서 “앱 실행 시 자동 저장되는 것이 의도인지”를 명시적으로 검증한다.

### 리스크 2 — 수집 성공 / 저장 실패를 하나로 뭉개면 UX가 애매해짐
설명:
- 시스템 정보는 보여줄 수 있지만 파일 저장만 실패할 수도 있다.

대응:
- 응답 구조에서 `items`와 `auto_saved_csv_path`를 분리한다.
- path가 없으면 UI에서 “수집 완료 / 자동 저장 실패”를 분리 표기한다.

### 리스크 3 — export 로직이 command 레이어에 묶여 재사용이 어색할 수 있음
설명:
- 현재 CSV 문자열 생성이 `commands/export.rs` 안에 있다 (`src-tauri/src/commands/export.rs:21-69`).

대응:
- 구현 시 command 내부 helper를 service/helper 함수로 분리하는 것을 우선 검토한다.
- 단, 과도한 구조 변경은 피하고 최소 diff 원칙을 유지한다.

### 리스크 4 — 문서와 코드가 다시 어긋날 수 있음
대응:
- 구현 완료 커밋 전에 `docs/unified-project-plan.md`, `docs/windows-manual-qa-checklist.md`, `CHANGELOG.md`를 함께 정리한다.

---

## 11. 검증 계획

### 정적 검증
- `npm run check`
- `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`

### 수동 검증
1. 앱 실행 직후 시스템 정보 자동 수집 완료 확인
2. `{exe_dir}/logs/`에 CSV 생성 확인
3. 파일명을 규칙과 비교
4. Excel / 메모장 열기 테스트
5. `재수집` 후 새 파일 생성 확인
6. 가상화 CSV 내보내기 기능이 그대로 동작하는지 확인

### QA 문서 반영 포인트
- `docs/windows-manual-qa-checklist.md:25-29`의 수동 export 문구를 자동 저장 기준으로 갱신

---

## 12. 구현 순서 추천

가장 안전한 구현 순서는 아래다.

1. `SystemInfoOutput` 타입 추가
2. 백엔드 자동 저장 helper 추가
3. `get_system_info` 반환 구조 변경
4. `App.svelte` 반영
5. `SystemInfoPanel.svelte` UI 정리
6. 문서/QA/CHANGELOG 정리
7. 정적 검증 실행

---

## 13. 검토 포인트

구현 전 리뷰에서 특히 확인받고 싶은 포인트는 아래 4개다.

1. **자동 저장을 앱 시작 시점에도 실행하는 정책을 그대로 가져갈지**
2. **수동 CSV 버튼을 완전히 제거할지**
3. **CSV 로직을 별도 service로 분리할지, 기존 command helper 재사용 수준으로 끝낼지**
4. **저장 위치 노출을 상태바만으로 충분히 볼지, 패널 카드에도 명시할지**

현재 추천안은 다음과 같다.

- 앱 시작 시 자동 저장: 유지
- 수동 CSV 버튼: 제거
- CSV 로직: 최소 diff로 재사용 가능하게만 분리
- 저장 위치 노출: 상태바 + 시스템 정보 패널 둘 다 표시

---

## 14. 한 줄 결론

Stage 4는 **시스템 정보 수집을 “수동 export 가능한 결과”에서 “자동 저장까지 포함된 완결된 작업”으로 승격**시키는 변경이며, 가장 적합한 구현 방식은 `get_system_info`가 수집 결과와 자동 저장 경로를 함께 반환하도록 계약을 바꾸는 것이다.
