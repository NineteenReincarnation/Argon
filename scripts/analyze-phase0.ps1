param(
    [string]$CsvPath = "run\argon\phase0-iris-uniforms.csv"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $CsvPath)) {
    throw "Phase 0 CSV not found: $CsvPath"
}

$Rows = @(Import-Csv $CsvPath)

if ($Rows.Count -eq 0) {
    throw "Phase 0 CSV contains no measurement rows: $CsvPath"
}

function Get-PhaseAMode($Row) {
    if ($null -eq $Row.phase_a_enabled -or $Row.phase_a_enabled -eq "") {
        return "unknown"
    }

    if ($Row.phase_a_enabled.ToString().ToLowerInvariant() -eq "true") {
        return "ON"
    }

    return "OFF"
}

function Write-Summary($InputRows, [string]$ModeLabel) {
    $RowsForMode = @($InputRows)

    if ($RowsForMode.Count -eq 0) {
        return
    }

    $TotalFrames = 0.0
    $WeightedEvaluations = 0.0
    $WeightedChanged = 0.0
    $WeightedStable = 0.0
    $WeightedPassPushes = 0.0
    $ActualChecks = 0.0
    $ActualUploads = 0.0
    $SimulatedChecks = 0.0
    $SimulatedRequired = 0.0
    $SimulatedAvoidable = 0.0
    $PhaseAFastPathSkips = 0.0
    $PhaseARevisionScans = 0.0
    $WeightedUpdateUs = 0.0
    $WeightedPushUs = 0.0

    foreach ($Row in $RowsForMode) {
        $Frames = [double]$Row.frames

        if ($Frames -le 0) {
            continue
        }

        $TotalFrames += $Frames
        $WeightedEvaluations += [double]$Row.evaluations_per_frame * $Frames
        $WeightedChanged += [double]$Row.changed_percent * $Frames
        $WeightedStable += [double]$Row.stable_percent * $Frames
        $WeightedPassPushes += [double]$Row.pass_pushes_per_frame * $Frames

        $ActualChecks += [double]$Row.actual_upload_checks_per_frame * $Frames
        $ActualUploads += [double]$Row.actual_uploads_per_frame * $Frames

        $SimulatedChecks += [double]$Row.simulated_upload_checks_per_frame * $Frames
        $SimulatedRequired += [double]$Row.simulated_required_per_frame * $Frames
        $SimulatedAvoidable += [double]$Row.simulated_avoidable_per_frame * $Frames
        $PhaseAFastPathSkips += [double]$Row.phase_a_fast_path_skips_per_frame * $Frames
        $PhaseARevisionScans += [double]$Row.phase_a_revision_scans_per_frame * $Frames

        $WeightedUpdateUs += [double]$Row.instrumented_update_us_per_frame * $Frames
        $WeightedPushUs += [double]$Row.instrumented_push_us_per_frame * $Frames
    }

    if ($TotalFrames -le 0) {
        return
    }

    function PerFrame([double]$Value) {
        return $Value / $TotalFrames
    }

    function Percent([double]$Numerator, [double]$Denominator) {
        if ($Denominator -le 0) {
            return 0.0
        }

        return ($Numerator * 100.0) / $Denominator
    }

    $Versions = $RowsForMode | ForEach-Object {
        "$($_.minecraft_version) | Iris $($_.iris_version) | Sodium $($_.sodium_version) | Argon $($_.argon_version)"
    } | Sort-Object -Unique

    $PipelineGenerations = $RowsForMode.pipeline_generation | Sort-Object -Unique

    Write-Output ""
    Write-Output "Phase A: $ModeLabel"
    Write-Output ("-" * (9 + $ModeLabel.Length))
    Write-Output "Windows: $($RowsForMode.Count)"
    Write-Output ("Frames: {0:N0}" -f $TotalFrames)
    Write-Output "Pipeline generations: $($PipelineGenerations -join ', ')"
    Write-Output ""
    Write-Output "Versions:"
    $Versions | ForEach-Object { Write-Output "  $_" }
    Write-Output ""
    Write-Output ("Evaluations/frame:              {0:N3}" -f (PerFrame $WeightedEvaluations))
    Write-Output ("Changed:                        {0:N3}%" -f ($WeightedChanged / $TotalFrames))
    Write-Output ("Stable:                         {0:N3}%" -f ($WeightedStable / $TotalFrames))
    Write-Output ("Pass pushes/frame:              {0:N3}" -f (PerFrame $WeightedPassPushes))
    Write-Output ""
    Write-Output ("Actual upload checks/frame:     {0:N3}" -f (PerFrame $ActualChecks))
    Write-Output ("Actual uploads/frame:           {0:N3}" -f (PerFrame $ActualUploads))
    Write-Output ("Simulated checks/frame:         {0:N3}" -f (PerFrame $SimulatedChecks))
    Write-Output ("Simulated required/frame:       {0:N3}" -f (PerFrame $SimulatedRequired))
    Write-Output ("Simulated avoidable/frame:      {0:N3}" -f (PerFrame $SimulatedAvoidable))
    Write-Output ("Simulated skip ratio:           {0:N3}%" -f (Percent $SimulatedAvoidable $SimulatedChecks))
    Write-Output ("Phase A fast skips/frame:       {0:N3}" -f (PerFrame $PhaseAFastPathSkips))
    Write-Output ("Phase A revision scans/frame:   {0:N3}" -f (PerFrame $PhaseARevisionScans))
    Write-Output ""
    Write-Output ("Instrumented update us/frame:   {0:N3}" -f (PerFrame $WeightedUpdateUs))
    Write-Output ("Instrumented push us/frame:     {0:N3}" -f (PerFrame $WeightedPushUs))
}

Write-Output ""
Write-Output "Argon Iris Uniform Phase 0 Summary"
Write-Output "=================================="
Write-Output "CSV: $CsvPath"

$Groups = $Rows | Group-Object { Get-PhaseAMode $_ }

foreach ($Group in $Groups | Sort-Object Name) {
    Write-Summary -InputRows $Group.Group -ModeLabel $Group.Name
}

Write-Output ""
Write-Output "Note: instrumented timings include measurement overhead and are not final speedup evidence."
