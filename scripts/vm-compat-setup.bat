@echo off
setlocal enabledelayedexpansion

REM VM Compatibility Tool - required setup for VM operation
REM Scope: Hyper-V/WSL features, hypervisor boot options, VBS and Core
REM Isolation registry values that must be off/disabled for a VM to run.
REM Optional/legacy items are NOT included here.
REM Source: src-tauri/src/commands/disable.rs, services/process_service.rs,
REM services/registry_manifest.rs
REM All screen output is plain ASCII English to avoid codepage/mojibake issues.

title VM Compatibility Setup

echo ================================================================
echo  VM Compatibility Setup
echo ================================================================
echo  This will disable Hyper-V and WSL features, turn off the
echo  hypervisor boot loader options, and disable VBS / Core
echo  Isolation registry values required for VM operation.
echo  Administrator privileges are required.
echo ================================================================
echo.

net session >nul 2>&1
if not %errorlevel%==0 (
    echo [ERROR] Administrator privileges are required.
    echo Right-click this file and choose "Run as administrator", then try again.
    echo.
    echo Press any key to close this window...
    pause >nul
    exit /b 1
)

echo --- Step 1: Windows features (DISM) ---
echo.
call :FeatureDisable Microsoft-Hyper-V-All
call :FeatureDisable Microsoft-Hyper-V
call :FeatureDisable Microsoft-Hyper-V-Tools-All
call :FeatureDisable Microsoft-Hyper-V-Management-PowerShell
call :FeatureDisable Microsoft-Hyper-V-Hypervisor
call :FeatureDisable Microsoft-Hyper-V-Services
call :FeatureDisable Microsoft-Hyper-V-Management-Clients
call :FeatureDisable Microsoft-Windows-Subsystem-Linux
call :FeatureDisable VirtualMachinePlatform

echo --- Step 2: Boot options (bcdedit) ---
echo.
call :BcdSet hypervisorlaunchtype off "hypervisorlaunchtype"
call :BcdSet vsmlaunchtype off "vsmlaunchtype"

echo --- Step 3: VBS registry values ---
echo.
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard" "EnableVirtualizationBasedSecurity" 0 "VBS EnableVirtualizationBasedSecurity (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard" "EnableVirtualizationBasedSecurity" 0 "VBS EnableVirtualizationBasedSecurity (ControlSet001)"
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard" "RequirePlatformSecurityFeatures" 0 "VBS RequirePlatformSecurityFeatures (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard" "RequirePlatformSecurityFeatures" 0 "VBS RequirePlatformSecurityFeatures (ControlSet001)"
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard" "Mandatory" 0 "VBS Mandatory (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard" "Mandatory" 0 "VBS Mandatory (ControlSet001)"
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\CredentialGuard" "Enabled" 0 "Credential Guard Enabled (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard\Scenarios\CredentialGuard" "Enabled" 0 "Credential Guard Enabled (ControlSet001)"
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\Lsa" "LsaCfgFlags" 0 "LSA LsaCfgFlags (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\Lsa" "LsaCfgFlags" 0 "LSA LsaCfgFlags (ControlSet001)"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "EnableVirtualizationBasedSecurity" 0 "VBS Policy EnableVirtualizationBasedSecurity"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "RequirePlatformSecurityFeatures" 0 "VBS Policy RequirePlatformSecurityFeatures"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "LsaCfgFlags" 0 "VBS Policy LsaCfgFlags"

echo --- Step 4: Core Isolation (HVCI) registry values ---
echo.
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity" "Enabled" 0 "HVCI Enabled (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity" "Enabled" 0 "HVCI Enabled (ControlSet001)"
call :RegSet "HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity" "Locked" 0 "HVCI Locked (CurrentControlSet)"
call :RegSet "HKLM\SYSTEM\ControlSet001\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity" "Locked" 0 "HVCI Locked (ControlSet001)"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "HypervisorEnforcedCodeIntegrity" 0 "Core Isolation Policy HypervisorEnforcedCodeIntegrity"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "HVCIEnabled" 0 "Core Isolation Policy HVCIEnabled"
call :RegSet "HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard" "HVCIMATRequired" 0 "Core Isolation Policy HVCIMATRequired"

echo ================================================================
echo  All tasks completed.
echo  A reboot is required for every change above to take full effect.
echo ================================================================
echo.
echo Press any key to close this window...
pause >nul
exit /b 0

REM ----------------------------------------------------------------
REM :FeatureDisable <FeatureName>
REM Disables one DISM optional feature and prints its before/after
REM state. /English forces English "State :" labels regardless of
REM the Windows display language, so no other language ever reaches
REM this script's parsing or output.
REM ----------------------------------------------------------------
:FeatureDisable
setlocal
set "FNAME=%~1"

set "FBEFORE="
for /f "tokens=1,* delims=:" %%A in ('dism.exe /English /online /get-featureinfo /featurename:%FNAME% 2^>nul ^| findstr /b /i "State"') do set "FBEFORE=%%B"
if not defined FBEFORE set "FBEFORE= Not available on this edition"

dism.exe /online /disable-feature /featurename:%FNAME% /norestart >nul 2>&1

set "FAFTER="
for /f "tokens=1,* delims=:" %%A in ('dism.exe /English /online /get-featureinfo /featurename:%FNAME% 2^>nul ^| findstr /b /i "State"') do set "FAFTER=%%B"
if not defined FAFTER set "FAFTER= Not available on this edition"

echo   %FNAME%
echo     Before:%FBEFORE%   After:%FAFTER%
echo.
endlocal
goto :eof

REM ----------------------------------------------------------------
REM :BcdSet <BcdOption> <TargetValue> <Label>
REM Sets a bcdedit boot option and prints its before/after value.
REM ----------------------------------------------------------------
:BcdSet
setlocal
set "BNAME=%~1"
set "BTARGET=%~2"
set "BLABEL=%~3"

set "BBEFORE="
for /f "tokens=1,*" %%A in ('bcdedit /enum "{current}" 2^>nul ^| findstr /b /i "%BNAME%"') do set "BBEFORE=%%B"
if not defined BBEFORE set "BBEFORE=Not set"

bcdedit /set %BNAME% %BTARGET% >nul 2>&1

set "BAFTER="
for /f "tokens=1,*" %%A in ('bcdedit /enum "{current}" 2^>nul ^| findstr /b /i "%BNAME%"') do set "BAFTER=%%B"
if not defined BAFTER set "BAFTER=Not set"

echo   %BLABEL%
echo     Before:%BBEFORE%   After:%BAFTER%
echo.
endlocal
goto :eof

REM ----------------------------------------------------------------
REM :RegSet <KeyPath> <ValueName> <TargetDword> <Label>
REM Only changes a value that already exists and differs from the
REM target - never creates a new value, matching the app's own
REM registry_manifest.rs behavior. Prints before/after either way.
REM ----------------------------------------------------------------
:RegSet
setlocal
set "RKEY=%~1"
set "RNAME=%~2"
set "RTARGET=%~3"
set "RLABEL=%~4"

set "RBEFORE="
for /f "tokens=3" %%V in ('reg query "%RKEY%" /v "%RNAME%" 2^>nul ^| findstr /i /c:"%RNAME%"') do set "RBEFORE=%%V"

if not defined RBEFORE (
    echo   %RLABEL%
    echo     Not set - skipped, no value created.
    echo.
    endlocal
    goto :eof
)

set /a RBEFOREDEC=%RBEFORE%
if %RBEFOREDEC%==%RTARGET% (
    echo   %RLABEL%
    echo     Already at target ^(%RBEFORE%^) - no change needed.
    echo.
    endlocal
    goto :eof
)

reg add "%RKEY%" /v "%RNAME%" /t REG_DWORD /d %RTARGET% /f >nul 2>&1

set "RAFTER="
for /f "tokens=3" %%V in ('reg query "%RKEY%" /v "%RNAME%" 2^>nul ^| findstr /i /c:"%RNAME%"') do set "RAFTER=%%V"

echo   %RLABEL%
echo     Before:%RBEFORE%   After:%RAFTER%
echo.
endlocal
goto :eof
