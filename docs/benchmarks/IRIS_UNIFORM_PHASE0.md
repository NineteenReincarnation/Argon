# Iris Custom Uniforms — Phase 0

Target stack:

- Minecraft 26.2
- Sodium 0.9.2
- Iris 1.11.4
- Spooklementary 2.0.4
- Argon 0.1.0 development line

## Validation status

The instrumentation currently has **CI compile validation only**. No in-game capture has been performed yet.

Until runtime testing is completed, numbers shown in this document are field definitions and decision rules, not measured results.

## Purpose

Phase 0 measures the existing Iris custom-uniform pipeline. It intentionally does **not** skip evaluations, skip uploads, alter shader settings, or modify shader-visible values.

It also simulates the proposed Phase A revision model in memory, without affecting Iris behavior. This estimates how many uploads would be required if each shader program only received a uniform when that uniform's value revision changed for that program.

## Instrumented values

Argon records:

- cached-uniform evaluations;
- evaluations whose value actually changed;
- custom-uniform pass pushes;
- actual Iris `pushIfChanged` checks/uploads;
- simulated per-program upload checks;
- simulated uploads required by a revision model;
- simulated uploads that would be avoidable;
- CPU time in `CustomUniforms.update()`;
- CPU time in `CustomUniforms.push(...)`.

A log report is emitted approximately every 10 seconds.

Important fields:

```text
stable
    Percentage of evaluated cached uniforms whose value did not change.

actualUploads/frame
    Uploads performed by the current Iris cached-uniform path.

simulatedRequired/frame
    Uploads the proposed per-program revision model would require.

simulatedAvoidable/frame
    Uploads that the simulation considers redundant for the same program and uniform revision.

simulatedSkip
    simulatedAvoidable / simulatedUploadChecks.
```

The simulated model is diagnostic only. It does not suppress any Iris upload.

## Pipeline invalidation

When Iris creates a new `CustomUniforms` instance, Argon clears the simulation's per-uniform and per-program revision state. This models the required invalidation behavior after pipeline/shader reconstruction.

## Runtime switches

Instrumentation is enabled by default in the current development build.

Disable it:

```text
-Dargon.instrumentation.irisUniforms=false
```

Change report interval:

```text
-Dargon.instrumentation.reportIntervalSeconds=20
```

## Initial scenarios

### Static outdoor

Stand still in a representative Overworld scene with stable weather.

### Continuous movement

Run and rotate the camera through changing terrain/biomes.

### Weather transition

Capture clear -> rain -> thunder transitions to exercise Spooklementary smoothing and weather uniforms.

### Entity stress

Use a repeatable entity-heavy scene with entity shadows enabled.

## Shader profiles

Record the exact local shader settings. Start with Medium and High; Very High is a useful stress case.

## Phase A decision gate

Do not implement upload deduplication merely because `simulatedAvoidable` is non-zero.

Proceed only when in-game captures show a repeatable combination of:

- a substantial simulated skip ratio;
- substantial avoided uploads per frame;
- measurable render-thread CPU cost in the custom-uniform push path;
- no indication that the simulated revision model misses required state transitions.

Raw benchmark captures do not need to live in Git. Representative results and reproduction settings should be committed.
