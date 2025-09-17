@echo off
chcp 65001 >nul
setlocal

echo ========================================
echo     빠른 빌드 (현재 버전, 기본 이름)
echo ========================================
echo.

:: 현재 디렉토리 확인
if not exist "VmCompatibilityTool.csproj" (
    echo 오류: VmCompatibilityTool.csproj 파일을 찾을 수 없습니다.
    echo 프로젝트 루트 디렉토리에서 실행해주세요.
    pause
    exit /b 1
)

echo 빌드를 시작합니다...
echo.

:: 이전 빌드 결과물 정리
if exist "bin\Release\net8.0-windows\win-x64\publish" (
    rmdir /s /q "bin\Release\net8.0-windows\win-x64\publish"
)

:: dotnet publish 실행 (기본 설정)
dotnet publish VmCompatibilityTool.csproj ^
    -c Release ^
    -r win-x64 ^
    --self-contained true ^
    -p:PublishSingleFile=true ^
    -p:EnableCompressionInSingleFile=true ^
    -p:IncludeNativeLibrariesForSelfExtract=true ^
    -o "bin\Release\net8.0-windows\win-x64\publish"

if %ERRORLEVEL% equ 0 (
    echo.
    echo ========================================
    echo 빌드 성공!
    echo ========================================
    echo.
    echo 생성된 파일: bin\Release\net8.0-windows\win-x64\publish\VM Compatibility Tool.exe

    :: release 폴더에 자동 복사
    if not exist "release" mkdir release
    copy "bin\Release\net8.0-windows\win-x64\publish\VM Compatibility Tool.exe" "release\"
    echo 파일이 release 폴더에도 복사되었습니다.

    echo.
    echo 빌드 완료! release 폴더를 확인하세요.
) else (
    echo.
    echo 빌드 실패! 오류 코드: %ERRORLEVEL%
)

echo.
timeout /t 3 /nobreak >nul