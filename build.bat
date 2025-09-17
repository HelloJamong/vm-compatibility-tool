@echo off
chcp 65001 >nul
setlocal

echo ========================================
echo     VM Compatibility Tool 빌드 스크립트
echo ========================================
echo.

:: 현재 디렉토리 확인
if not exist "VmCompatibilityTool.csproj" (
    echo 오류: VmCompatibilityTool.csproj 파일을 찾을 수 없습니다.
    echo 프로젝트 루트 디렉토리에서 실행해주세요.
    pause
    exit /b 1
)

:: 사용자 입력 받기
echo 빌드 옵션을 선택하세요:
echo 1. 현재 버전으로 빌드
echo 2. 버전 업데이트 후 빌드
echo.
set /p choice="선택 (1 또는 2): "

if "%choice%"=="2" (
    echo.
    echo 새 버전 정보를 입력하세요:
    set /p new_version="버전 (예: 1.2.2): "

    if "!new_version!"=="" (
        echo 오류: 버전을 입력해주세요.
        pause
        exit /b 1
    )

    :: 버전 업데이트
    echo.
    echo 버전을 !new_version!로 업데이트 중...

    :: csproj 파일의 버전 업데이트
    powershell -Command ^
        "$content = Get-Content 'VmCompatibilityTool.csproj' -Raw; " ^
        "$content = $content -replace '<AssemblyVersion>[^<]*</AssemblyVersion>', '<AssemblyVersion>!new_version!.0</AssemblyVersion>'; " ^
        "$content = $content -replace '<FileVersion>[^<]*</FileVersion>', '<FileVersion>!new_version!.0</FileVersion>'; " ^
        "Set-Content 'VmCompatibilityTool.csproj' -Value $content -NoNewline"

    :: MainWindow.xaml.cs 파일의 버전 업데이트
    powershell -Command ^
        "$content = Get-Content 'MainWindow.xaml.cs' -Raw; " ^
        "$content = $content -replace 'VersionTextBlock\.Text = \"\"v[^\"\"]*\"\"', 'VersionTextBlock.Text = \"\"v!new_version!\"\"'; " ^
        "$content = $content -replace 'VM Compatibility Tool v[^\"\"]*', 'VM Compatibility Tool v!new_version!'; " ^
        "Set-Content 'MainWindow.xaml.cs' -Value $content -NoNewline"

    echo 버전 업데이트 완료: !new_version!
    echo.
)

:: 출력 파일명 설정
echo 출력 파일명을 설정하세요:
set /p output_name="파일명 (확장자 제외, 기본값: VM-Compatibility-Tool): "

if "%output_name%"=="" (
    set output_name=VM-Compatibility-Tool
)

:: 빌드 실행
echo.
echo ========================================
echo 빌드를 시작합니다...
echo 출력 파일: %output_name%.exe
echo ========================================
echo.

:: 이전 빌드 결과물 정리
if exist "bin\Release\net8.0-windows\win-x64\publish" (
    echo 이전 빌드 결과물을 정리 중...
    rmdir /s /q "bin\Release\net8.0-windows\win-x64\publish"
)

:: dotnet publish 실행
dotnet publish VmCompatibilityTool.csproj ^
    -c Release ^
    -r win-x64 ^
    --self-contained true ^
    -p:PublishSingleFile=true ^
    -p:EnableCompressionInSingleFile=true ^
    -p:IncludeNativeLibrariesForSelfExtract=true ^
    -p:AssemblyName="%output_name%" ^
    -o "bin\Release\net8.0-windows\win-x64\publish"

:: 빌드 결과 확인 (ERRORLEVEL은 변할 수 있으므로 미리 저장)
set BUILD_RESULT=%ERRORLEVEL%

echo.
if %BUILD_RESULT% equ 0 (
    echo ========================================
    echo 빌드 성공!
    echo ========================================
    echo.

    :: 빌드된 파일 정보 표시
    if exist "bin\Release\net8.0-windows\win-x64\publish\%output_name%.exe" (
        echo 생성된 파일: bin\Release\net8.0-windows\win-x64\publish\%output_name%.exe

        :: release 폴더에 자동 복사
        if not exist "release" mkdir release
        copy "bin\Release\net8.0-windows\win-x64\publish\%output_name%.exe" "release\" >nul
        echo 파일이 release 폴더에 복사되었습니다.
    ) else (
        echo 경고: 예상된 출력 파일을 찾을 수 없습니다.
    )

) else (
    echo ========================================
    echo 빌드 실패!
    echo ========================================
    echo 오류 코드: %BUILD_RESULT%
    echo.
    echo 가능한 원인:
    echo - .NET 8 SDK가 설치되지 않음
    echo - 프로젝트 파일 손상
    echo - 디스크 공간 부족
    echo.
)

echo.
echo 아무 키나 눌러 종료하세요...
pause >nul