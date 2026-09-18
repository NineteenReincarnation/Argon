# Iris Custom Uniforms — Phase 0

Target stack:

- Minecraft 26.2
- Sodium 0.9.2
- Iris 1.11.4
- Spooklementary 2.0.4
- Argon 0.1.0 development line

## Purpose

Phase 0 measures the existing Iris custom-uniform pipeline. It intentionally does **not** skip evaluations, skip uploads, alter shader settings, or modify shader-visible values.

The instrumentation is intended to answer whether Phase A upload deduplication is worth implementing.

## Instrumented values

Argon records:

- cached-uniform evaluations;
- evaluations whose value actually changed;
- custom-uniform pass pushes;
- `pushIfChanged` checks;
- checks that currently perform an upload;
- CPU time in `CustomUniforms.update()`;
- CPU time in `CustomUniforms.push(...)`.

A log report is emitted approximately every 10 seconds.

Example shape:

```text
[Argon] [Phase 0][Iris uniforms] frames=... eval/frame=... changed=...%, stable=...%, passPush/frame=..., uploadChecks/frame=..., actualUploads/frame=..., uploaded=...%, updateUs/frame=..., pushUs/frame=...
```

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

Stand still in a representative Overworld scene with stable weather. This is expected to expose uniforms that are evaluated and uploaded despite stable values.

### Continuous movement

Run and rotate the camera through changing terrain/biomes. This checks the high-change case.

### Weather transition

Capture clear -> rain -> thunder transitions to exercise Spooklementary smoothing and weather uniforms.

### Entity stress

Use a repeatable entity-heavy scene under a Spooklementary profile with entity shadows enabled.

## Shader profiles

Record the exact local shader settings. The initial baseline should include at least Medium and High; Very High is a useful stress case.

## Phase A decision gate

Do not implement upload deduplication only because redundant work exists.

Proceed when the measurements show a repeatable combination of:

- a substantial stable-value ratio;
- a substantial number of upload checks/uploads per frame;
- measurable render-thread CPU cost in the custom-uniform push path.

Raw benchmark captures do not need to live in Git. Record representative results and reproduction settings here or in a dedicated result document.
