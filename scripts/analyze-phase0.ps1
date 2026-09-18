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
$WeightedUpdateUs = 0.0
$WeightedPushUs = 0.0

foreach ($Row in $Rows) {
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

    $WeightedUpdateUs += [double]$Row.instrumented_update_us_per_frame * $Frames
    $WeightedPushUs += [double]$Row.instrumented_push_us_per_frame * $Frames
}

if ($TotalFrames -le 0) {
    throw "Phase 0 CSV contains no rows with a positive frame count."
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

$Versions = $Rows | ForEach-Object {
    "$($_.minecraft_version) | Iris $($_.iris_version) | Sodium $($_.sodium_version) | Argon $($_.argon_version)"
} | Sort-Object -Unique

$PipelineGenerations = $Rows.pipeline_generation | Sort-Object -Unique

Write-Host ""
Write-Host "Argon Iris Uniform Phase 0 Summary"
Write-Host "=================================="
Write-Host "CSV: $CsvPath"
Write-Host "Windows: $($Rows.Count)"
Write-Host ("Frames: {0:N0}" -f $TotalFrames)
Write-Host "Pipeline generations: $($PipelineGenerations -join ', ')"
Write-Host ""
Write-Host "Versions:"
$Versions | ForEach-Object { Write-Host "  $_" }
Write-Host ""
Write-Host ("Evaluations/frame:              {0:N3}" -f (PerFrame $WeightedEvaluations))
Write-Host ("Changed:                        {0:N3}%" -f ($WeightedChanged / $TotalFrames))
Write-Host ("Stable:                         {0:N3}%" -f ($WeightedStable / $TotalFrames))
Write-Host ("Pass pushes/frame:              {0:N3}" -f (PerFrame $WeightedPassPushes))
Write-Host ""
Write-Host ("Actual upload checks/frame:     {0:N3}" -f (PerFrame $ActualChecks))
Write-Host ("Actual uploads/frame:           {0:N3}" -f (PerFrame $ActualUploads))
Write-Host ("Simulated checks/frame:         {0:N3}" -f (PerFrame $SimulatedChecks))
Write-Host ("Simulated required/frame:       {0:N3}" -f (PerFrame $SimulatedRequired))
Write-Host ("Simulated avoidable/frame:      {0:N3}" -f (PerFrame $SimulatedAvoidable))
Write-Host ("Simulated skip ratio:           {0:N3}%" -f (Percent $SimulatedAvoidable $SimulatedChecks))
Write-Host ""
Write-Host ("Instrumented update us/frame:   {0:N3}" -f (PerFrame $WeightedUpdateUs))
Write-Host ("Instrumented push us/frame:     {0:N3}" -f (PerFrame $WeightedPushUs))
Write-Host ""
Write-Host "Note: instrumented timings include measurement overhead and are not final speedup evidence."
