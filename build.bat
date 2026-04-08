@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo   VM Compatibility Tool (Tauri) 빌드
echo ========================================
echo.

:: 프로젝트 루트 확인
if not exist "src-tauri\Cargo.toml" (
    echo 오류: src-tauri\Cargo.toml 파일을 찾을 수 없습니다.
    echo 프로젝트 루트 디렉토리에서 실행해주세요.
    pause
    exit /b 1
)

:: npm 의존성 확인
if not exist "node_modules" (
    echo npm 의존성을 설치합니다...
    npm ci
    if errorlevel 1 (
        echo npm 설치 실패
        pause
        exit /b 1
    )
    echo.
)

:: 빌드 유형 선택
echo 빌드 유형을 선택하세요:
echo 1. 포터블 EXE  ^(번들 없음^)
echo 2. NSIS 인스톨러
echo.
set /p build_type="선택 (1 또는 2, 기본값: 1): "
if "%build_type%"=="" set build_type=1

echo.
echo ========================================
echo 빌드를 시작합니다...
echo ========================================
echo.

if "%build_type%"=="2" (
    echo NSIS 인스톨러 빌드 중...
    npm run tauri build -- --bundles nsis
) else (
    echo 포터블 EXE 빌드 중...
    npm run tauri build -- --bundles none
)

set BUILD_RESULT=%ERRORLEVEL%

echo.
if %BUILD_RESULT% equ 0 (
    echo ========================================
    echo 빌드 성공!
    echo ========================================
    echo.

    if not exist "release" mkdir release

    if "%build_type%"=="2" (
        echo 인스톨러 위치: src-tauri\target\release\bundle\nsis\
        xcopy /y "src-tauri\target\release\bundle\nsis\*.exe" "release\" >nul 2>&1
    ) else (
        set SRC=src-tauri\target\release\vm-compatibility-tool.exe
        set DST=release\VM-Compatibility-Tool.exe
        if exist "!SRC!" (
            copy /y "!SRC!" "!DST!" >nul
            echo 출력: !DST!
        ) else (
            echo 경고: 출력 EXE를 찾을 수 없습니다: !SRC!
        )
    )

    echo 파일이 release 폴더에 복사되었습니다.
) else (
    echo ========================================
    echo 빌드 실패! 오류 코드: %BUILD_RESULT%
    echo ========================================
    echo.
    echo 가능한 원인:
    echo - Rust 툴체인 미설치  ^(rustup 설치 필요^)
    echo - Node.js / npm 미설치
    echo - Tauri CLI 미설치  ^(cargo install tauri-cli --version "^^2"^)
    echo - WebView2 관련 빌드 오류
    echo.
)

echo.
echo 아무 키나 눌러 종료하세요...
pause >nul
