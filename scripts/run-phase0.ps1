param(
    [switch]$EnablePhaseA
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$ShaderPath = Join-Path $RepoRoot "References\26.2\third-party\spooklementary\shaders"

$CsvPath = Join-Path $RepoRoot "run\argon\phase0-iris-uniforms.csv"
if (Test-Path $CsvPath) {
    $ArchiveDir = Join-Path $RepoRoot "run\argon\archive"
    New-Item -ItemType Directory -Force -Path $ArchiveDir | Out-Null
    $Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $ArchivedCsv = Join-Path $ArchiveDir "phase0-iris-uniforms-$Stamp.csv"
    Move-Item -Path $CsvPath -Destination $ArchivedCsv
    Write-Host "Archived previous Phase 0 CSV to: $ArchivedCsv"
}

if (-not (Test-Path $ShaderPath)) {
    Write-Host "Spooklementary reference is not initialized. Initializing Git submodules..."
    git submodule update --init --recursive

    if ($LASTEXITCODE -ne 0) {
        throw "git submodule update failed with exit code $LASTEXITCODE."
    }
}

if (-not (Test-Path $ShaderPath)) {
    throw "Spooklementary shaders directory is still missing: $ShaderPath"
}

Write-Host ""
Write-Host "Starting Argon Phase 0 development client..."
Write-Host "Target: Minecraft 26.2 / Iris 1.11.4 / Sodium 0.9.2 / Spooklementary 2.0.4"
Write-Host ""

$GradleArgs = @("runClient", "-Pargon_dev_shader_stack=true")

if ($EnablePhaseA) {
    $GradleArgs += "-Pargon_phase_a=true"
    Write-Host "Experimental Phase A uniform upload deduplication: ENABLED"
} else {
    Write-Host "Experimental Phase A uniform upload deduplication: disabled"
}

& .\gradlew.bat @GradleArgs

$GradleExit = $LASTEXITCODE
$LatestLog = Join-Path $RepoRoot "run\logs\latest.log"
$Summary = Join-Path $RepoRoot "run\argon-phase0-summary.txt"

if (Test-Path $LatestLog) {
    $Patterns = @(
        "Argon client baseline:",
        "Iris uniform Phase 0 instrumentation:",
        "Enabling Minecraft 26.2 Iris Phase 0 integration",
        "[Phase 0][Iris uniforms]"
    )

    $Lines = Get-Content $LatestLog | Where-Object {
        $Line = $_
        $Patterns | Where-Object { $Line.Contains($_) }
    }

    $Lines | Set-Content -Path $Summary -Encoding UTF8

    Write-Host ""
    Write-Host "Argon Phase 0 log summary:"
    Write-Host "--------------------------"

    if ($Lines.Count -gt 0) {
        $Lines | ForEach-Object { Write-Host $_ }
        Write-Host ""
        Write-Host "Saved summary to: $Summary"

        if (Test-Path $CsvPath) {
            Write-Host "Structured Phase 0 CSV: $CsvPath"
        }
    } else {
        Write-Warning "No Argon Phase 0 lines were found in latest.log."
    }
} else {
    Write-Warning "Minecraft latest.log was not found at: $LatestLog"
}

if ($GradleExit -ne 0) {
    exit $GradleExit
}
