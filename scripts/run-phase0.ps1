param(
    [ValidateSet("1.11.0", "1.11.1", "1.11.2", "1.11.4")]
    [string]$IrisVersion = "1.11.4",
    [switch]$EnablePhaseA,
    [switch]$PerformanceMode,
    [switch]$Jfr
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$VersionPair = switch ($IrisVersion) {
    "1.11.0" {
        @{
            Iris = "1.11.0+26.2-fabric"
            Sodium = "mc26.2-0.9.0-fabric"
            SodiumDisplay = "0.9.0"
        }
    }
    "1.11.1" {
        @{
            Iris = "1.11.1+26.2-fabric"
            Sodium = "mc26.2-0.9.0-fabric"
            SodiumDisplay = "0.9.0"
        }
    }
    "1.11.2" {
        @{
            Iris = "1.11.2+26.2-fabric"
            Sodium = "mc26.2-0.9.1-fabric"
            SodiumDisplay = "0.9.1"
        }
    }
    "1.11.4" {
        @{
            Iris = "1.11.4+26.2-fabric"
            Sodium = "mc26.2-0.9.2-fabric"
            SodiumDisplay = "0.9.2"
        }
    }
}

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
Write-Host "Starting Argon Iris development client..."
Write-Host "Target: Minecraft 26.2 / Iris $IrisVersion / Sodium $($VersionPair.SodiumDisplay) / Spooklementary 2.0.4"
Write-Host ""

$GradleArgs = @(
    "runClient",
    "-Pargon_dev_shader_stack=true",
    "-Pdev_iris_version=$($VersionPair.Iris)",
    "-Pdev_sodium_version=$($VersionPair.Sodium)"
)

if ($EnablePhaseA) {
    $GradleArgs += "-Pargon_phase_a=true"
    Write-Host "Experimental Phase A uniform upload deduplication: ENABLED"
} else {
    Write-Host "Experimental Phase A uniform upload deduplication: disabled"
}

if ($PerformanceMode) {
    $GradleArgs += "-Pargon_instrumentation=false"
    Write-Host "Detailed Phase 0 instrumentation: disabled (performance mode)"
} else {
    Write-Host "Detailed Phase 0 instrumentation: enabled"
}

$JfrAbsolutePath = $null

if ($Jfr) {
    $JfrDir = Join-Path $RepoRoot "run\argon\jfr"
    New-Item -ItemType Directory -Force -Path $JfrDir | Out-Null

    $Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $Mode = if ($EnablePhaseA) { "phase-a" } else { "baseline" }
    $JfrName = "$Mode-iris-$IrisVersion-$Stamp.jfr"
    $JfrRelativePath = "argon/jfr/$JfrName"
    $JfrAbsolutePath = Join-Path $JfrDir $JfrName

    $GradleArgs += "-Pargon_jfr_file=$JfrRelativePath"
    Write-Host "JFR recording: $JfrAbsolutePath"
}

& .\gradlew.bat @GradleArgs

$GradleExit = $LASTEXITCODE
$LatestLog = Join-Path $RepoRoot "run\logs\latest.log"
$Summary = Join-Path $RepoRoot "run\argon-phase0-summary.txt"

if (Test-Path $LatestLog) {
    $Patterns = @(
        "Argon client baseline:",
        "Iris uniform Phase 0 instrumentation:",
        "Iris uniform Phase A deduplication:",
        "Enabling Minecraft 26.2 Iris Phase 0 integration",
        "[Phase 0][Iris uniforms]",
        "[Phase A][Iris uniforms]"
    )

    $Lines = Get-Content $LatestLog | Where-Object {
        $Line = $_
        $Patterns | Where-Object { $Line.Contains($_) }
    }

    $Lines | Set-Content -Path $Summary -Encoding UTF8

    Write-Host ""
    Write-Host "Argon Iris log summary:"
    Write-Host "-----------------------"

    if ($Lines.Count -gt 0) {
        $Lines | ForEach-Object { Write-Host $_ }
        Write-Host ""
        Write-Host "Saved summary to: $Summary"

        if (Test-Path $CsvPath) {
            Write-Host "Structured Phase 0 CSV: $CsvPath"
        } elseif ($PerformanceMode) {
            Write-Host "No Phase 0 CSV expected: detailed instrumentation was disabled."
        }
    } else {
        Write-Warning "No Argon Iris lines were found in latest.log."
    }
} else {
    Write-Warning "Minecraft latest.log was not found at: $LatestLog"
}

if ($JfrAbsolutePath -and (Test-Path $JfrAbsolutePath)) {
    Write-Host "JFR saved to: $JfrAbsolutePath"
}

if ($GradleExit -ne 0) {
    exit $GradleExit
}
