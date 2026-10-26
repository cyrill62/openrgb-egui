<#
.SYNOPSIS
    Captures the PimpBox RGB GUI pages against the local mock OpenRGB server.

.PARAMETER OutputDir
    Directory for the generated PNG files. Defaults to docs/screenshots.
#>
[CmdletBinding()]
param([string] $OutputDir = 'docs/screenshots')

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Port = 26742
$AppExe = Join-Path $RepoRoot 'target\debug\pimpbox-rgb.exe'
$MockExe = Join-Path $RepoRoot 'target\debug\examples\mock_server.exe'
$hadShots = Test-Path Env:PIMPBOX_RGB_SHOTS
$oldShots = $env:PIMPBOX_RGB_SHOTS
$mockProcess = $null
$mockOut = Join-Path $env:TEMP "pimpbox-rgb-mock-$PID.out.log"
$mockErr = Join-Path $env:TEMP "pimpbox-rgb-mock-$PID.err.log"

Push-Location $RepoRoot
try {
    & cargo build --features screenshot --bin pimpbox-rgb --example mock_server
    if ($LASTEXITCODE -ne 0) { throw 'Cargo build failed.' }

    $OutputDir = if ([System.IO.Path]::IsPathRooted($OutputDir)) {
        [System.IO.Path]::GetFullPath($OutputDir)
    } else {
        [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $OutputDir))
    }
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    $mockProcess = Start-Process -FilePath $MockExe -ArgumentList @([string]$Port) -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput $mockOut -RedirectStandardError $mockErr

    $ready = $false
    for ($i = 0; $i -lt 50; $i++) {
        if ($mockProcess.HasExited) {
            $details = if (Test-Path $mockErr) { Get-Content $mockErr -Raw } else { '' }
            throw "Mock server exited before it started. $details"
        }
        if (Select-String -Path $mockOut -SimpleMatch "mock OpenRGB server listening on 127.0.0.1:$Port" -Quiet -ErrorAction SilentlyContinue) {
            $ready = $true
            break
        }
        Start-Sleep -Milliseconds 100
    }
    if (-not $ready) { throw 'Mock server did not start listening.' }

    $env:PIMPBOX_RGB_SHOTS = $OutputDir
    & $AppExe "127.0.0.1:$Port"
    if ($LASTEXITCODE -ne 0) { throw 'Screenshot app failed.' }

    $shots = @(Get-ChildItem -Path $OutputDir -Filter '*.png' -File)
    if ($shots.Count -eq 0) { throw "No PNG files were written to $OutputDir." }
    $shots | Format-Table Name, Length
} finally {
    if ($null -ne $mockProcess -and -not $mockProcess.HasExited) {
        Stop-Process -Id $mockProcess.Id -Force
        $mockProcess.WaitForExit()
    }
    if ($hadShots) { $env:PIMPBOX_RGB_SHOTS = $oldShots }
    else { Remove-Item Env:PIMPBOX_RGB_SHOTS -ErrorAction SilentlyContinue }
    Remove-Item $mockOut, $mockErr -Force -ErrorAction SilentlyContinue
    Pop-Location
}
