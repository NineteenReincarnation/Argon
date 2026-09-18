# Iris Custom Uniforms — Phase 0

Target stack:

- Minecraft 26.2
- Sodium 0.9.2
- Iris 1.11.4
- Spooklementary 2.0.4
- Argon 0.1.0 development line

Pinned Iris source used for the 26.2 audit:

```text
f61d950f556f3962d0e2e29c270bc1865572b35d
```

## Validation status

The instrumentation currently has **CI compile validation only**. No in-game capture has been performed yet.

Until runtime testing is completed, field descriptions and decision rules below are not measured results.

## Static source audit

The pinned Iris 1.11.4 source confirms the assumptions behind Phase 0:

- `CachedUniform.update()` calls `doUpdate()`, discards its boolean result, and then sets `changed = true`.
- `CustomUniforms.locationMap` maps pass/program objects to the cached uniforms and GL locations used by that pass.
- `mapholderToPass(...)` replaces the temporary builder key with the actual program/pass object.
- Iris renderers retain their `Program` / `ComputeProgram` objects across frames.
- program reconstruction creates new program objects rather than mutating one object's GL identity in place in the audited paths.
- `CustomUniforms.optimise()` has one pipeline-initialization call site in the pinned source and is used as Argon's revision-state reset point.
- the Iris 1.11.4 release runtime version is `1.11.4+mc26.2`.

The cached-uniform audit also confirms that the four instrumented `doUpdate()` implementations cover the scalar types plus the shared vector/matrix path in the pinned source.

These observations are source-level validation only. Runtime behavior still needs game testing.

## Purpose

Phase 0 measures the existing Iris custom-uniform pipeline. It intentionally does **not** skip evaluations, skip uploads, alter shader settings, or modify shader-visible values.

It also simulates the proposed Phase A revision model in memory, without affecting Iris behavior. This estimates how many uploads would be required if each shader program only received a uniform when that uniform's value revision changed for that program.

## Measurement window

When Iris reaches `CustomUniforms.optimise()` for a newly built pipeline, Argon clears the simulation state and enters a warm-up period.

Default:

```text
5 seconds
```

During warm-up, revision/program state is still tracked so the simulation reaches steady state, but counters and timings are not included in the report.

Override:

```text
-Dargon.instrumentation.warmupSeconds=10
```

Set to `0` to disable warm-up.

## Instrumented values

Argon records:

- cached-uniform evaluations;
- evaluations whose value actually changed;
- custom-uniform pass pushes;
- actual Iris `pushIfChanged` checks/uploads;
- simulated per-program upload checks;
- simulated uploads required by a revision model;
- simulated uploads that would be avoidable;
- tracked uniform/program counts;
- instrumented wall-clock time around `CustomUniforms.update()`;
- instrumented wall-clock time around `CustomUniforms.push(...)`.

A log report is emitted approximately every 10 seconds after warm-up.

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

instrumentedUpdateUs/frame
instrumentedPushUs/frame
    Diagnostic timing collected while detailed instrumentation hooks are active.
    These values include instrumentation overhead and are NOT final benchmark evidence.
```

For actual performance claims, use A/B builds and an external profiler/benchmark method. Do not subtract these diagnostic timings and call the difference an Argon speedup.

## Runtime switches

Instrumentation is enabled by default in the current development build.

Disable:

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


## Machine-readable output

Each completed measurement interval is appended to:

```text
<game directory>/argon/phase0-iris-uniforms.csv
```

For the Loom development client this is normally:

```text
run/argon/phase0-iris-uniforms.csv
```

The CSV includes the pipeline generation, installed Argon/Minecraft/Iris/Sodium versions, frame count, uniform/program counts, actual and simulated upload rates, skip ratio, and instrumented update/push timing.

A pipeline/shader rebuild increments `pipeline_generation`, so measurements before and after a reload can be separated without guessing from timestamps.

CSV write failures never stop rendering. Argon logs the error once and disables further CSV output for that session.
