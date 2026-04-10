param(
    [Parameter(Mandatory = $true)]
    [string[]]$Files
)

$ErrorActionPreference = "Stop"

function Resolve-SignToolPath {
    $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($cmd) {
        return $cmd.Source
    }

    $kitRoot = "C:\Program Files (x86)\Windows Kits\10\bin"
    if (-not (Test-Path $kitRoot)) {
        throw "signtool.exe 를 찾을 수 없습니다. Windows SDK가 설치되어 있는지 확인하세요."
    }

    $candidate = Get-ChildItem $kitRoot -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*\x64\signtool.exe" } |
        Sort-Object FullName -Descending |
        Select-Object -First 1

    if (-not $candidate) {
        throw "Windows Kits 경로에서 signtool.exe 를 찾을 수 없습니다."
    }

    return $candidate.FullName
}

$certBase64 = $env:WINDOWS_CODESIGN_PFX_BASE64
$certPassword = $env:WINDOWS_CODESIGN_PFX_PASSWORD
$timestampUrl = $env:WINDOWS_CODESIGN_TIMESTAMP_URL

if ([string]::IsNullOrWhiteSpace($certBase64)) {
    throw "WINDOWS_CODESIGN_PFX_BASE64 secret 이 비어 있습니다."
}

if ([string]::IsNullOrWhiteSpace($certPassword)) {
    throw "WINDOWS_CODESIGN_PFX_PASSWORD secret 이 비어 있습니다."
}

$signTool = Resolve-SignToolPath
$tempPfxPath = Join-Path $env:RUNNER_TEMP "vmct-codesign.pfx"

try {
    $normalizedBase64 = ($certBase64 -replace "\s", "")
    [System.IO.File]::WriteAllBytes(
        $tempPfxPath,
        [System.Convert]::FromBase64String($normalizedBase64)
    )

    foreach ($file in $Files) {
        if (-not (Test-Path $file)) {
            throw "서명 대상 파일을 찾을 수 없습니다: $file"
        }

        $resolvedFile = (Resolve-Path $file).Path
        $signArgs = @(
            "sign",
            "/fd", "SHA256",
            "/f", $tempPfxPath,
            "/p", $certPassword,
            "/d", "VM Compatibility Tool",
            "/v"
        )

        if (-not [string]::IsNullOrWhiteSpace($timestampUrl)) {
            $signArgs += @("/td", "SHA256", "/tr", $timestampUrl)
        }

        $signArgs += $resolvedFile

        Write-Host "Signing: $resolvedFile"
        & $signTool @signArgs
        if ($LASTEXITCODE -ne 0) {
            throw "signtool sign 실패: $resolvedFile (exit $LASTEXITCODE)"
        }

        $signature = Get-AuthenticodeSignature -FilePath $resolvedFile
        if (-not $signature.SignerCertificate) {
            throw "서명 인증서를 읽지 못했습니다: $resolvedFile"
        }
        if ($signature.Status -in @("NotSigned", "HashMismatch")) {
            throw "서명 검증 실패 상태($($signature.Status)): $resolvedFile"
        }

        Write-Host "Signer Subject: $($signature.SignerCertificate.Subject)"
        Write-Host "Runner Verification Status: $($signature.Status)"
        if ($signature.Status -eq "NotTrusted") {
            Write-Warning "자가 서명/사설 CA 인증서는 runner 또는 사용자 PC의 신뢰 저장소에 루트 인증서가 없으면 NotTrusted 로 보일 수 있습니다."
        }
    }
}
finally {
    if (Test-Path $tempPfxPath) {
        Remove-Item $tempPfxPath -Force -ErrorAction SilentlyContinue
    }
}
