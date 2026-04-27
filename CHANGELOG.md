# Changelog

## [Release-v26.04.10] - 2026-04-27

> 버전 표기는 날짜가 아니라 `연도.월.n번째 버전` 규칙을 따른다.
> `v26.04.10`은 2026년 4월의 10번째 정식 Release 버전을 의미한다.

### 주요 변경사항 요약

#### 최초 자동 점검 진행 표시 개선
- 프로그램 실행 직후 점검 모달 하단의 진행 문구가 실제 작업과 무관하게 순환 표시되던 문제 수정
- 초기 점검 작업을 실제 실행 순서 기준 단계로 표시
  - 점검 준비 중
  - 시스템 정보 수집 중
  - 가상화 설정 점검 중
  - 설치된 프로그램 목록 수집 중
  - 점검 결과 CSV 저장 중
  - 점검 완료
- 진행률을 실제 단계별 완료 지점에 맞게 보정
  - 시스템 정보 수집 완료 시 약 35%
  - 가상화 설정 점검 완료 시 약 65%
  - 설치된 프로그램 목록 수집 완료 시 약 90%
  - CSV 저장 중 99%까지 진행 후 완료 시 100%
- 기존처럼 95% 근처에서 오래 멈춰 보이는 체감 문제를 줄이고, 현재 실행 중인 단계가 명확히 보이도록 개선

---

## [Release-v26.04.09] - 2026-04-27

> 버전 표기는 날짜가 아니라 `연도.월.n번째 버전` 규칙을 따른다.
> `v26.04.09`는 2026년 4월의 9번째 정식 Release 버전을 의미한다.

### 주요 변경사항 요약

#### 설치된 프로그램 / 앱 목록 CSV 품질 개선
- `Programs.csv`에 Windows 내부 Appx 시스템 패키지가 과도하게 포함되던 문제 수정
  - GUID 형태 패키지 이름 제외
  - `MicrosoftWindows.*` 계열 내부 패키지 제외
  - ShellExperienceHost, StartMenuExperienceHost, LockApp, OOBE, PrintDialog 등 설정 앱의 “설치된 앱” 목록에 보이지 않는 시스템 구성 패키지 제외
- Appx 수집 범위를 `Get-AppxPackage -AllUsers`에서 현재 사용자 기준 `Get-AppxPackage`로 조정
  - 다른 사용자 또는 과거 상태에 남은 패키지가 CSV에 섞이는 가능성 축소
- Appx 패키지 표시명을 `Get-StartApps` / manifest 표시명 기준으로 보정
  - 가능한 경우 패키지 ID 대신 사용자가 보는 앱 이름에 가까운 값 저장
- Appx 게시자 DN 문자열을 사람이 읽기 쉬운 제조사명으로 정규화
  - 예: `CN=Microsoft Corporation, O=Microsoft Corporation, ...` → `Microsoft Corporation`

---

## [Release-v26.04.08] - 2026-04-27

> 버전 표기는 날짜가 아니라 `연도.월.n번째 버전` 규칙을 따른다.
> `v26.04.08`은 2026년 4월의 8번째 정식 Release 버전을 의미한다.

### 주요 변경사항 요약

#### 최초 실행 자동 저장 CSV 확장
- 프로그램 최초 실행 시 자동 생성되는 CSV 파일을 기존 2개에서 총 3개로 확장
  - `YYMMDD_HHMMSS_{HostName}-SystemInfo.csv`
  - `YYMMDD_HHMMSS_{HostName}-reg.csv`
  - `YYMMDD_HHMMSS_{HostName}-Programs.csv`

#### 설치된 프로그램 / 앱 목록 CSV 추가
- 제어판 → 프로그램 목록 기준으로 확인 가능한 Win32 설치 프로그램 목록 수집 추가
  - `HKLM` / `HKCU` Uninstall 레지스트리 확인
  - 64-bit / 32-bit(`WOW6432Node`) 설치 항목 모두 확인
- Windows 설정 → 앱 → 설치된 앱 기준으로 확인 가능한 Appx/UWP 앱 목록 수집 추가
  - `Get-AppxPackage -AllUsers` 기반 수집
- 새 CSV에 아래 3개 값을 저장
  - 설치된 프로그램 이름
  - 제조사
  - 날짜
- 레지스트리 `InstallDate`의 `YYYYMMDD` 값은 `YYYY-MM-DD` 형식으로 정규화
- Windows가 설치 날짜를 제공하지 않는 앱은 날짜 값을 빈 값으로 저장
- 중복 설치 항목은 프로그램 이름 + 제조사 기준으로 정리 후 이름순으로 정렬

#### 자동 점검 화면 반영
- 최초 자동 점검 진행 상태에 “설치된 프로그램 목록 수집 중...” 메시지 추가
- 점검 완료 모달의 저장 파일 목록에 `Programs.csv` 파일명을 함께 표시
- 상태바 busy 상태에 설치 프로그램 목록 수집 작업을 포함

---

## [Release-v26.04.07] - 2026-04-22

### 주요 변경사항 요약

#### USB 저장장치 타입 오표시 수정
- USB 드라이브가 "알 수 없음"으로 표시되던 문제 수정
- 원인: `MSFT_PhysicalDisk`가 USB 장치에 `MediaType = 0`을 반환할 때 `Some(Unknown)`으로 감싸져 모델명 키워드 폴백이 실행되지 않던 구조적 버그
- 수정 내용
  - 인터페이스 타입이 `USB`이면 WMI/모델명 판별 이전에 즉시 `USB 저장장치`로 확정
  - WMI가 `Unknown`을 반환한 경우에도 모델명 키워드 폴백이 실행되도록 분기 개선
  - `DiskType::Usb` 변형 추가 — 표시값 `"USB 저장장치"`

---

## [Release-v26.04.06] - 2026-04-22

### 주요 변경사항 요약

#### 운영체제 에디션 오표시 수정
- Windows 11에서도 `ProductName` 레지스트리가 `"Windows 10 Pro"`를 반환하는 Microsoft 버그 대응
- `ProductName`의 OS 이름 접두사를 제거하고 빌드 번호 기반 `os_name`과 조합하여 정확한 에디션 표시
  - 수정 전: `Windows 10 Pro` (빌드 26200 시스템에서 잘못 표시)
  - 수정 후: `Windows 11 Pro`

#### 설치 언어 수집 오류 수정
- `InstallLanguage` 레지스트리 키가 없는 시스템에서 "알 수 없음"으로 표시되던 문제 수정
- `InstallLanguage`가 비어있을 경우 `SYSTEM\...\Nls\Language\Default` 키로 폴백하여 시스템 로케일 수집

#### Windows 업데이트 수집 성능 개선
- 진행률이 95%에서 2분 이상 정체되던 문제 수정
- 전체 업데이트 이력(`$count`) 전체 로드에서 최신 300건으로 제한
  - Windows Defender 정의 업데이트 등 수천 건 누적 이력 전체 조회 방지
  - 최신 300건은 3개월치 KB 업데이트를 모두 포함하기에 충분

---

## [Release-v26.04.05] - 2026-04-22

### 주요 변경사항 요약

#### 운영체제 수집 항목 확장
- 기존 이름 / 버전 / 빌드 / 에디션 4개 항목에 3개 추가
  - **아키텍처**: `SYSTEM\...\Environment\PROCESSOR_ARCHITECTURE` — `x64 (AMD64)` / `ARM64` / `x86 (32비트)` 로 변환
  - **설치 날짜**: `InstallDate` (DWORD Unix timestamp) → `YYYY-MM-DD` 형식으로 변환
  - **설치 언어**: `InstallLanguage` (LCID hex) → 한국어 / 영어 (미국) / 일본어 / 중국어 (간체/번체) 매핑, 미지원 코드는 원문 표시
- 모든 항목은 기존 CSV 자동 저장에 그대로 포함됨

#### Windows 업데이트 이력 수집 추가
- 최근 90일 이내 설치 성공한 업데이트의 KB 카탈로그 번호를 CSV에 기록
- PowerShell `Microsoft.Update.Session` COM (WUA) 기반 — `Win32_QuickFixEngineering` 대비 누락 없는 전체 이력 조회
- CSV 출력 형태: `Windows 업데이트 | KB5034441 | 2025-03-15` (항목당 1행)
- 3개월 내 기록 없음 → `최근 3개월 | 업데이트 기록 없음` 1행, COM 오류 시 오류 메시지 기록

---

## [Release-v26.04.04] - 2026-04-20

### 주요 변경사항 요약

#### 레지스트리 매니페스트 확장
- `RunAsPPL`, `RunAsPPLBoot` (LSA 보호 프로세스) 항목을 `InspectOnly`로 추가
  - `LsaCfgFlags`와 별개의 LSA 보호 플래그 — 직접 비활성화는 위험도가 높아 점검 전용으로 등록
- `EnableSecureLaunch` (Secure Launch / DRTM) 항목을 `ExcludedLegacy`로 추가
  - VM 호환성에 영향을 줄 수 있는 Secure Launch 설정을 선택적 조치 후보로 등록

#### vsmlaunchtype 비활성화 지원
- bcdedit `vsmlaunchtype off` 추가 — VSM(Virtual Secure Mode) 시작 유형을 Hyper-V 비활성화 흐름에서 함께 처리
- 가상화 점검 결과에 `VSM 시작 유형 (vsmlaunchtype)` 항목 추가 — BCD 값이 활성 상태이면 `Hyperv` 그룹 조치 필요로 표시
- Hyper-V 비활성화 로그에 `✓ vsmlaunchtype off` 결과 기록

#### 조직 관리 장치 감지 (Azure AD / MDM)
- 가상화 점검 중 Azure AD 조인 또는 MDM 기업 등록을 감지해 `org_control_check` 항목으로 표시
- 가상화 점검 결과 표에 `조직 관리 장치` 행 추가 — 연결 유형(AAD 조인 / MDM 등록 / 둘 다)을 파란 계열 배지로 표시
- 비활성화 패널에 조직 관리 감지 시 정보 배너 추가
  - "비활성화 후 재부팅 시 VBS 설정이 정책으로 재적용될 수 있습니다 — IT 관리자 확인 권장"
  - 조치 시작 버튼을 막지 않는 정보 전용 경고

#### QA 체크리스트 최신화
- 현재 소스 기준으로 `docs/windows-manual-qa-checklist.md` 전면 검토 및 갱신
  - 없어진 항목 제거 (시스템 정보 패널 상세 표시, 메뉴 배지 수동 갱신 등)
  - 자동 점검/자동 저장 흐름에 맞게 항목 재작성
  - `hypervisorlaunchtype off` 로그 확인 항목 추가
  - 창 크기 · 최대화 불가 조건 명시

---

## [Release-v26.04.03] - 2026-04-10

### 주요 변경사항 요약

#### 시작 점검 완료 화면 가독성 개선
- 점검 완료 후 안내 문구를 2줄로 명확히 분리
  - `최초 실행에 따른 하드웨어 및 가상화 설정 검사가 완료되었습니다.`
  - `아래 항목에 대한 조치 필요 여부를 확인하세요.`
- 완료 화면 내부 여백/아이콘/버튼 높이를 조정해 화면 밀도를 개선
- footer 버전 정보가 스크롤 없이 보이도록 시작 화면 레이아웃을 보정

#### 창 높이 조정
- 고정 창 높이를 `660x640` 기준에서 더 여유 있게 보정
  - `height`: `700`
  - `minHeight`: `620`
- 완료 상태에서 점검 요약, 경고, 버튼, footer가 한 화면 안에 더 안정적으로 들어오도록 조정

#### 배포 워크플로 유지보수
- GitHub Actions의 Node 20 deprecation 경고 대응
  - `actions/checkout@v5`
  - `actions/setup-node@v5`
- `softprops/action-gh-release` 의존을 제거하고 `gh release` CLI 기반으로 배포 절차를 정리
- beta / release 워크플로 모두 최신 운영 흐름에 맞게 정리

---

## [Release-v26.04.02] - 2026-04-10

### 주요 변경사항 요약

#### 레지스트리 추가 선택 조치
- 자동 조치 기본 목록에는 포함되지 않던 legacy VBS/코어 격리 레지스트리 항목을
  실행 전 체크박스로 선택할 수 있도록 확장
- 선택한 추가 항목만 별도 작업으로 실행되며, 로그/백업에도 함께 반영
- `InspectOnly` 항목과 Windows Hello for Business 경고는 계속 읽기 전용/선행 확인 항목으로 유지

#### 비활성화 UI 개선
- 비활성화 패널에 “추가 레지스트리 조치는 실행 전 선택 가능” 안내 추가
- 비활성화 확인 모달에 추가 선택 가능한 레지스트리 후보 목록 표시
- 선택된 자동 조치 + 추가 조치 수를 함께 집계해 실행 전 검토 가능

#### 저장소 정리
- `docs/`, `.claude/` 폴더를 Git 추적 대상에서 제거
- `.gitignore`에 재추적 방지 규칙 추가

#### 배포 정책 정리
- 정식 Release 워크플로 산출물 파일명을 `vm-compatibility-tool.exe`로 고정
- 앱 내부 표시 버전은 계속 하단 footer에서 확인 가능
- beta 워크플로는 기존처럼 버전 포함 파일명 정책 유지

#### 문서 보강
- README 기능 정의 섹션에 Hyper-V / WSL / VBS / 코어 격리 조치 대상 설명 추가
- 자동 조치 대상 / 수동 선택 가능 레지스트리 / 읽기 전용 항목별 역할과 위험도 정리

---

## [Release-v26.04.01] - 2026-04-10

### 주요 변경사항 요약

#### 자동 점검 및 CSV 자동 저장
- 앱 시작 시 시스템 정보 + 가상화 점검을 자동 수행
- 점검 결과를 `vmc_logs/` 하위에 CSV 2개로 자동 저장
  - `YYMMDD_HHMMSS_{HostName}-SystemInfo.csv`
  - `YYMMDD_HHMMSS_{HostName}-reg.csv`

#### 시작 점검 화면 (`InspectionSummaryModal`)
- 진행률 바 + 퍼센트 + 현재 수집 항목 실시간 표시
- 점검 완료 후 조치 필요 항목 요약 표시
- Windows Hello for Business 감지 시 경고 카드 표시 및 조치 시작 버튼 비활성화

#### 비활성화 조치 모달 (`DisableActionModal`)
- warning → running → complete 3단계 전체화면 UI
  - **warning**: 조치 예정 항목 목록 + 주의사항 표시
  - **running**: 프로그레스바 + 퍼센트 + 현재 진행 항목 실시간 표시
  - **complete**: 조치 완료 후 재부팅 분기
    - 예 → 5초 후 자동 재부팅
    - 아니요 → 수동 재부팅 안내 후 닫기

#### 로그 및 백업 자동 저장
- 비활성화 실행 후 운영 로그 자동 저장: `vmc_logs/YYMMDD_HHMMSS_{HostName}.log`
- 레지스트리 수정 전 백업 자동 저장: `vmc_backup/YYMMDD_HHMMSS_backup.reg`

#### Registry Manifest
- `DisableWrite` / `InspectOnly` / `ExcludedLegacy` 분류 기반 점검·조치 단일 source of truth
- 선택적 비활성화: 가상화 점검 결과 기반 필요 그룹만 실행

#### UI / 창 설정
- 창 크기 `660x640` 고정 (`resizable: false`, `maximizable: false`)
- 닫기 버튼과 Footer 사이 여백 추가
- 시작 점검 화면 footer에 실제 앱 버전 표시

#### 배포 인프라
- GitHub Actions beta / release 워크플로에 내부 코드 서명 단계 추가
- nightly 자동 빌드 워크플로 제거

---

## [beta-v26.04.01.0024] - 2026-04-10

### Added
- 비활성화 조치 전용 전체화면 모달 (`DisableActionModal`) 추가
  - **warning 단계**: 조치 예정 항목 목록과 주의사항 표시, "조치 시작" / "취소" 버튼
  - **running 단계**: 프로그레스바 + 퍼센트 + 현재 진행 항목 실시간 표시
  - **complete 단계**: 조치 결과 요약 및 저장 파일명 표시
    - "예 — 지금 재부팅 (5초 후)" 선택 시 즉시 재부팅 예약
    - "아니요 — 나중에 재부팅" 선택 시 수동 재부팅 안내 후 닫기

---

## [beta-v26.04.01.0023] - 2026-04-10

### Changed
- 레지스트리 백업 파일 저장 위치를 `vmc_logs`에서 `vmc_backup` 폴더로 분리
  - 운영 로그: `vmc_logs/YYMMDD_HHMMSS_{HostName}.log` (기존 유지)
  - 레지스트리 백업: `vmc_backup/YYMMDD_HHMMSS_backup.reg` (신규)

---

## [beta-v26.04.01.0022] - 2026-04-10

### Added
- 시작 점검 완료 화면에 Windows Hello for Business 경고 추가
  - WHfB 감지 시 경고 카드 표시 ("비활성화 후 재시도" 안내)
  - WHfB 감지 상태에서는 `조치 시작` 버튼 비활성화

---

## [beta-v26.04.01.0021] - 2026-04-10

### Changed
- 앱 창 최대화 버튼 비활성화
  - `maximizable: false` 적용으로 창 크기를 늘릴 수 없도록 제한
- 시작 점검 화면 닫기 버튼과 Footer 사이 여백 추가
  - `.actions`에 `margin-bottom: 24px` 적용
- 자동 저장 로그 폴더명 변경: `logs` → `vmc_logs`
  - 다른 프로그램 생성 폴더와 구분 가능하도록 변경
- 자동 저장 CSV 파일명 규칙 변경
  - 시스템 정보: `YYMMDD_HHMMSS_{HostName}-SystemInfo.csv`
  - 가상화 점검 결과: `YYMMDD_HHMMSS_{HostName}-reg.csv`

---

## [beta-v26.04.01.0020] - 2026-04-10

### Fixed
- 축소한 시작 화면/앱 창이 여전히 크기 조절 가능하던 문제 수정
  - 앱 창을 고정 크기로 동작하도록 `resizable: false` 반영

---

## [beta-v26.04.01.0019] - 2026-04-10

### Changed
- 앱 기본 창 폭을 기존 대비 약 1/3 축소
  - `980x640` → `660x640`
  - 최소 폭도 `820` → `560` 으로 조정
- 시작 점검 화면 진행률 보정 로직 재조정
  - 중간 단계에서 `82%` 부근에 멈춘 것처럼 보이던 구간을 완화
  - 수집/저장 중 더 자연스럽게 계속 증가하도록 staged progress 로직 수정

---

## [beta-v26.04.01.0018] - 2026-04-10

### Fixed
- 시작 점검 화면 `닫기` 버튼이 실제 앱 종료로 이어지지 않던 문제 수정
  - 백엔드 종료 커맨드를 통해 창이 닫히도록 정리
- 시작 점검 진행 상태가 어떤 항목을 처리 중인지 보이지 않던 문제 수정
  - 프로그레스바 하단에 현재 수집/저장 중인 작업명을 표시
- 레지스트리 상세 정보가 `|` 구분자로 뭉쳐 보여 가독성이 떨어지던 문제 수정
  - 줄바꿈 기반 표시로 변경

### Changed
- 자동 저장 점검 결과를 단일 CSV가 아닌 2개 파일로 분리
  - 시스템 정보 CSV
  - 가상화 점검 결과 CSV
- 시작 화면에서 저장된 결과 파일명을 복수 파일 기준으로 표시하도록 조정

---

## [beta-v26.04.01.0017] - 2026-04-10

### Fixed
- 시작 점검 화면에서 `닫기` 선택 시 앱 창이 실제로 종료되도록 수정
- 시작 점검 진행률이 중간 구간에서 멈춘 것처럼 보이던 문제를 단계형 진행률 보정으로 완화
- 자동 저장 점검 결과 CSV에 시스템 정보와 가상화 점검 결과를 함께 포함하도록 수정

### Changed
- 시작 화면 footer 높이를 축소하고 버전 정보만 표시하도록 정리
- 시작 화면 좌우 여백과 메인 진입 후 컨텐츠 여백을 축소해 실제 UI 폭을 더 넓게 사용

---

## [beta-v26.04.01.0016] - 2026-04-10

### Fixed
- 시작 점검 화면에서 `닫기` 클릭 시 기존 레거시 UI로 돌아가던 문제 수정
  - 이제 창 자체를 종료하도록 변경
- 시작 점검 진행률이 `55%` 근처에서 멈춘 것처럼 보이던 문제 수정
  - 단계 기반 타이머 진행률로 변경해 점검/저장 흐름이 계속 보이도록 개선
- 자동 저장되는 점검 결과 CSV에 시스템 하드웨어 정보가 누락되던 문제 수정
  - `[시스템 정보]`
  - `[가상화 점검 결과]`
  두 섹션을 모두 포함하도록 변경

### Changed
- 시작 화면 footer에 실제 앱 버전 표시
- 시작 화면 및 메인 컨텐츠의 불필요한 좌우 여백 축소

---

## [beta-v26.04.01.0015] - 2026-04-10

### Changed
- 시작 점검 화면을 현재 디자인 샘플 기준으로 추가 정렬
  - summary card와 버튼 구조를 샘플 비율에 맞게 조정
  - 아이콘 / 타이포 / 간격 스타일을 더 직접적인 CSS 기반으로 고정
- 기존에 누적돼 있던 beta prerelease/tag를 정리하고 최신 beta 릴리즈 한 개 기준으로 다시 발행하도록 정리

---

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
