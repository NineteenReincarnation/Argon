# Argon Project Management

## Current release line

**Minecraft 26.2 — Iris / Spooklementary optimization baseline**

Minecraft target: **26.2**

Current validation state is tracked in `docs/VALIDATION_STATUS.md`. At present, the project has CI build validation but no in-game validation.

## Version management

Argon uses `main` as the single active development branch.

All development, fixes, documentation, tooling, validation work, and release preparation are committed directly to `main`. Historical `mc/<version>` and `dev/<version>/...` branches are not part of the active workflow.

Project progression is tracked by the Argon version number rather than by release branches.

### Version format

Argon uses:

```text
X.Y.Z+<minecraft-version>
```

For the current Minecraft line:

```text
X.Y.Z+26.2
```

The fields have project-specific meanings:

- **X — major content version.** X is currently `1`. It changes only when the project owner explicitly decides that Argon has entered a new major version.
- **Y — release version.** Y identifies the current Argon release line. When development moves to the next release line, Y increments.
- **Z — development iteration.** Z increments by 1 for each development iteration within the same Y release line.
- **+26.2 — Minecraft target.** This records the target Minecraft version without changing the meaning of X/Y/Z.

When Y increments, Z resets to `0`.

Examples:

```text
1.0.0+26.2
1.0.1+26.2
1.0.2+26.2
...
1.1.0+26.2
1.1.1+26.2
...
```

Temporary validation states such as compile/package/structure/runtime/visual/performance verification are tracked separately and are not encoded as `dev`, `alpha`, `beta`, or `rc` in the version string by default.

Compatibility code may still use explicit Minecraft-version packages or reference directories when upstream structure actually differs. This is an implementation boundary, not a separate Git development line.

## Runtime compatibility gate

A version-specific Mixin fails closed when its external target version has not been validated.

For the current Iris work, the 26.2 integration is gated to the exact Fabric metadata baseline:

- Minecraft `26.2`
- Iris `1.11.4+mc26.2`

The human-facing Iris release is 1.11.4; its 26.2 Fabric build appends `+mc26.2` to the runtime version string.

Unknown Iris versions do not automatically receive the patch. Compatibility can be widened only after source review and runtime validation.

## Commit policy

Use short scoped prefixes: `build:`, `chore:`, `docs:`, `perf:`, `fix:`, `refactor:`, and `test:`.

Avoid mixing unrelated work in one commit.

## Current optimization phases

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

Starts only if Phase 0 demonstrates meaningful redundant uploads.

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

Compile-only validation does not satisfy runtime, visual, or performance gates.

## Release process

1. Finish scoped issues for the current Y release line on `main`.
2. Run runtime correctness and compatibility checks.
3. Record benchmark evidence.
4. Finalize the changelog.
5. Confirm the target `main` commit passes the required validation gates.
6. Tag that `main` commit using its full Argon version, for example `v1.0.12+26.2`.
7. Publish the CI-built JAR.
8. When starting the next release line, increment Y and reset Z before new development iterations begin.

X changes only by an explicit project-owner decision.
