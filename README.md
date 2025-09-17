# VM Compatibility Tool

VM 사용을 위한 시스템 최적화 도구

## 📋 프로젝트 개요

VM Compatibility Tool은 Windows 시스템에서 VM을 원활하게 사용하기 위해 필요한 시스템 정보 확인 및 최적화 작업을 자동화하는 WPF 애플리케이션입니다.

## 🎯 주요 기능

### 1. **시스템 사양 체크**
- **운영체제 정보**: Windows 버전, 빌드 번호, 에디션 확인
- **CPU 정보**: 프로세서 모델, 제조사, 코어/스레드 수, 최대 클럭 속도
- **메모리 정보**: 총 물리적 메모리, 사용 가능한 메모리
- **디스크 정보**: 드라이브별 용량 정보, SSD/HDD 구분
- **가상화 지원**: 하드웨어 가상화 활성화 여부, Hyper-V 상태, VBS 상태
- **부팅 정보**: 마지막 부팅 시간, 시스템 가동 시간

### 2. **VBS 및 Hyper-V 비활성화**
가상머신 성능 최적화를 위해 Windows 보안 기능을 비활성화:

- **Hyper-V 제거**: 모든 Hyper-V 관련 Windows 기능 비활성화
- **WSL2 제거**: Windows Subsystem for Linux 및 Virtual Machine Platform 비활성화  
- **VBS 비활성화**: 가상화 기반 보안(Virtualization-based Security) 비활성화
- **코어 격리 비활성화**: 하이퍼바이저 코드 무결성(HVCI) 및 관련 보안 기능 비활성화
- **자동 재부팅**: 작업 완료 후 즉시 재부팅 또는 나중에 재부팅 옵션 제공

## 🖥️ 시스템 요구사항

- **운영체제**: Windows 10 이상 (64비트)
- **프레임워크**: .NET 8 Runtime (Self-contained 빌드의 경우 불필요)
- **권한**: 시스템 정보 조회 및 Windows 기능 변경을 위한 관리자 권한

## 🚀 사용법

### 실행파일 다운로드
1. 릴리즈 페이지에서 `VM Compatibility Tool.exe` 다운로드
2. 관리자 권한으로 실행

### 소스코드에서 빌드

#### 자동 빌드 스크립트 사용 (권장)
```bash
# 프로젝트 클론
git clone https://github.com/HelloJamong/vm-compatibility-tool.git
cd vm-compatibility-tool

# 옵션 1: 대화형 빌드 (버전 업데이트 + 파일명 설정 가능)
build.bat

# 옵션 2: 빠른 빌드 (현재 버전, 기본 파일명)
quick-build.bat
```

**빌드 스크립트 기능:**
- `build.bat`: 버전 자동 업데이트, 출력 파일명 설정, 대화형 인터페이스
  - 현재 버전으로 빌드 또는 새 버전으로 업데이트 후 빌드 선택 가능
  - 출력 파일명 커스터마이징 (예: `MyTool`, `VM-Tool-v2` 등)
  - 빌드 완료 후 release 폴더 복사 및 탐색기 열기 옵션
  - VmCompatibilityTool.csproj와 MainWindow.xaml.cs의 버전 정보 자동 동기화
- `quick-build.bat`: 설정 없이 빠른 빌드, release 폴더에 자동 복사
  - 현재 버전 유지, 기본 파일명(`VM Compatibility Tool.exe`) 사용
  - 3초 후 자동 종료로 빠른 개발 워크플로우 지원

#### 수동 빌드
```bash
# 개발용 실행
dotnet run --project VmCompatibilityTool.csproj

# 배포용 빌드 (단일 실행파일)
dotnet publish VmCompatibilityTool.csproj -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true
```

## ⚠️ 주의사항

- **관리자 권한 필요**: 시스템 정보 수집 및 Windows 기능 변경을 위해 관리자 권한이 필요합니다
- **보안 기능 비활성화**: VBS 및 Hyper-V 비활성화는 시스템 보안 수준을 낮출 수 있습니다
- **재부팅 필요**: 변경사항 적용을 위해 시스템 재부팅이 필요합니다
- **백업 권장**: 시스템 변경 전 중요 데이터 백업을 권장합니다

## 🛠️ 기술 스택

- **Language**: C# 12
- **Framework**: .NET 8
- **UI Framework**: WPF (Windows Presentation Foundation)
- **Dependencies**: 
  - System.Management (WMI 정보 수집)
- **Build**: Single-file self-contained executable

## 🤖 개발 도구

이 프로젝트는 **Claude Code**를 통해 제작되었습니다.
- AI 지원 개발 환경에서 설계 및 구현
- 코드 품질 및 안정성 최적화
- 자동화된 오류 처리 및 예외 관리 시스템 구축

**최종 업데이트**: 2025-09-17
