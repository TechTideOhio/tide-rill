#!/usr/bin/env pwsh
# tide-rill: TideOS Eval Kit Tool 5 runner wrapper

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$BinRelease = Join-Path $ScriptDir "target\release\tide-rill.exe"
$BinDebug = Join-Path $ScriptDir "target\debug\tide-rill.exe"

if (-not (Test-Path $BinRelease) -and -not (Test-Path $BinDebug)) {
    Write-Host "[tide-rill] Compiling release binary..." -ForegroundColor Cyan
    Push-Location $ScriptDir
    try {
        cargo build --release
    } finally {
        Pop-Location
    }
}

$Bin = if (Test-Path $BinRelease) { $BinRelease } else { $DebugBin }
& $Bin @Arguments
exit $LASTEXITCODE
