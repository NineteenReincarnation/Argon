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


## Local aggregation

On Windows, aggregate all rows currently in the CSV with:

```powershell
.\scripts\analyze-phase0.ps1
```

The script uses frame-weighted aggregation and reconstructs total simulated upload checks/required/avoidable work from each interval. It does not apply an arbitrary go/no-go threshold.


## Phase A implementation status

An experimental per-program revision deduplicator is now present in the development branch but is disabled by default.

The revision belongs to each Iris `CachedUniform`. A revision increments only when Iris' own `doUpdate()` reports a value change. Each pass/program remembers the last revision it actually received. A missing program/uniform entry always uploads.

Pipeline initialization through `CustomUniforms.optimise()` clears the per-program upload history.

This directly models the limitation documented in Iris 1.11.4's `CachedUniform.update()`: the cached uniform cannot safely clear the global `changed` flag because one uniform may still need to be uploaded to another program.

Phase A does not mutate Iris' original `changed` field. When disabled, the original `CustomUniforms.push()` path runs unchanged.


### Phase A hot-path implementation

The active Phase A path stores program identities in a fastutil `Reference2ObjectOpenHashMap` and per-program uniform revisions in `Reference2LongOpenHashMap`.

This preserves identity semantics while avoiding boxed `Long` values on every upload check.

The implementation is fail-closed at runtime. If the expected Iris `Object2IntMap` or Argon-injected cached-uniform state is missing, Phase A disables itself for the remainder of the session, clears its state, logs the reason, and returns control to Iris' original `CustomUniforms.push()` path.


The local analyzer groups rows by `phase_a_enabled`. Baseline and experimental upload counts are therefore never averaged into one number.

The pinned Iris 1.11.4 pipeline calls `CustomUniforms.update()` once during the new-frame path before the normal render-pass pushes. This supports the Phase A revision model: one custom-uniform revision snapshot is shared by the program uploads that follow that frame update.


### Phase A program-epoch fast path

Phase A also maintains one global custom-uniform change epoch.

Every real cached-uniform value change increments this epoch. Each shader program records the epoch observed after its last successful custom-uniform push.

If a program is used again while its recorded epoch still equals the global epoch, no custom uniform can have changed since that program was synchronized. Argon therefore cancels the Iris custom-uniform push without iterating the program's uniform map.

If the epoch differs, Argon falls back to the per-uniform revision comparison and then marks the program synchronized to the new epoch.

The epoch is reset together with all per-program state when the custom-uniform pipeline is rebuilt.


### Program remapping safety

Phase A does not trust Java pass identity alone. Each pass state also remembers the exact Iris location-map object used for that pass.

If Iris reuses a Java pass object but rebuilds/remaps its uniform locations, the location-map identity changes. Argon then discards that pass's uploaded-revision history and treats the next use as a first use, forcing the required uploads.

CI now checks the Phase A `CachedUniform.update()` / `CustomUniforms.push()` bytecode assumptions for every published Iris 1.11.x Fabric build in the Minecraft 26.2 compatibility matrix, not only the primary 1.11.4 baseline.


Phase A diagnostics additionally report:

```text
phaseAFastSkip/frame
phaseAScan/frame
```

The first counts program pushes that were resolved by the epoch fast path without walking the uniform map. The second counts pushes that entered the per-uniform revision scan.


### Phase A consistency checks

The Phase 0 simulator now uses the same pass + location-map identity invalidation rule as Phase A.

When Phase A is active, Argon compares the number of uploads the experimental path actually performs with the number required by the independent revision simulation for the same interval. A mismatch is logged as a warning.

If Phase A disables itself during a measurement window, Argon discards that mixed window before returning to Iris' original path so the CSV does not blend experimental and baseline behavior.


### Combined Phase A fast path

The current implementation combines both safety and fast-path state:

- pass identity selects the program state;
- location-map identity invalidates that state if Iris remaps the same pass;
- a global change epoch skips the entire map when no custom uniform changed since the program's last synchronized push;
- per-uniform revisions handle frames where at least one custom uniform changed;
- a first use or remap always enters the revision scan and uploads every required uniform.

The epoch is an optimization only. Per-uniform revisions remain the correctness mechanism.


### Phase A changed-set incremental path

The pinned Iris 1.11.4 source updates the custom-uniform order through `CustomUniforms.update()`, once on the audited new-frame path before render-pass pushes.

Argon uses that boundary to retain a reusable list of uniforms whose `doUpdate()` actually changed during the current update sequence.

For a program that was synchronized in the immediately preceding update sequence:

- if the changed set is smaller than that program's mapped uniform set, Argon checks only the changed uniforms;
- if the changed set is not smaller, Argon uses the full revision scan;
- if the program missed one or more update sequences, Argon uses the full revision scan;
- repeated pushes in the same update sequence use the O(1) synchronized-update fast path.

This keeps the incremental path conservative when a program is not rendered every frame.

Diagnostics distinguish `phaseAFastSkip/frame`, `phaseAIncremental/frame`, and `phaseAFullScan/frame`.


## Instrumentation versus performance mode

The detailed Phase 0 simulator deliberately performs work that the optimized fast path intends to remove. It is for correctness/opportunity measurement, not final A/B performance claims.

For final Phase A performance runs, disable it with:

```text
-Dargon.instrumentation.irisUniforms=false
```

or use the repository helper:

```powershell
.\scripts\run-phase0.ps1 -EnablePhaseA -PerformanceMode
```

Compare against the same command without `-EnablePhaseA`.


### Phase A three-tier push path

The experimental deduplicator now uses one conservative hierarchy:

1. **Fast skip** — the program is already synchronized in the current update sequence, or the immediately following update changed no custom uniforms.
2. **Incremental changed-set push** — the program was synchronized in the previous update and fewer custom uniforms changed than the program maps.
3. **Full revision scan** — first use, skipped update sequences, a remapped location map, or a changed set large enough that a full scan is cheaper.

The changed-set path is only used when the program was synchronized in the immediately previous update. A program that was not rendered for one or more update sequences always falls back to a full revision scan.

If a cached uniform reports a change outside the audited `CustomUniforms.update()` boundary, Argon invalidates all program synchronization state rather than trusting an incomplete changed set.
