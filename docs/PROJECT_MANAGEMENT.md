# Argon Project Management

## Current release line

**v0.1.0 — Iris / Spooklementary optimization baseline**

Minecraft target: **26.2**

The first release line establishes Argon's build, compatibility, profiling, and patch-isolation workflow while investigating the Iris custom-uniform pipeline.

## Version format

Argon uses semantic versioning for the mod itself and includes the Minecraft target as build metadata.

Release: `0.1.0+mc26.2`

Development build: `0.1.0-dev.1+mc26.2`

Release candidate: `0.1.0-rc.1+mc26.2`

A future Minecraft version gets its own reference baseline and compatibility work; Minecraft versions are never mixed inside one reference directory.

## Branch policy

- `main`: buildable integration branch; no knowingly broken experiments.
- `dev/<version>-<scope>`: active release/feature development.
- `fix/<scope>`: focused regressions or compatibility fixes.

There is no permanent `develop` branch. Long-running work stays scoped and is merged through a pull request.

## Commit policy

Use short scoped prefixes: `build:`, `chore:`, `docs:`, `perf:`, `fix:`, `refactor:`, and `test:`.

Avoid mixing unrelated work in one commit.

## v0.1.0 phases

### Phase 0 — Instrumentation

Measure the existing Iris custom-uniform path without changing shader output.

Required data:

- cached-uniform evaluations per frame;
- actual value-change rate;
- custom-uniform pass pushes per frame;
- uniform upload checks and actual uploads per frame;
- CPU time in `CustomUniforms.update()`;
- CPU time in `CustomUniforms.push(...)`.

### Phase A — Upload deduplication

Starts only if Phase 0 demonstrates meaningful redundant uploads. The intended design is per-uniform revision plus per-program uploaded revision, preserving exact shader-visible state.

### Phase B — Dependency-aware evaluation

Starts only if expression evaluation remains meaningful after Phase A. Stateful/time-dependent expressions such as `smooth(...)` must not be treated as pure cached expressions.

### Phase C — Adjacent Iris shadow allocation work

Only pursued if JFR identifies repeatable allocation/GC cost in the shadow path.

## Merge gates

A performance patch is mergeable when:

1. the problem is reproducible;
2. reference-mod overlap has been checked;
3. the patch is independently disableable where practical;
4. the target metric improves measurably;
5. visual/game behavior remains correct;
6. the relevant compatibility matrix passes;
7. CI passes.

For rendering work, use `References/26.2/SHADER_COMPATIBILITY.md`.

## Release process

1. Finish scoped issues.
2. Run correctness and compatibility checks.
3. Record benchmark evidence.
4. Finalize the changelog.
5. Change `mod_version` from a development version to the release version.
6. Merge the release PR to `main`.
7. Tag the merge commit `vX.Y.Z`.
8. Publish the CI-built JAR.
