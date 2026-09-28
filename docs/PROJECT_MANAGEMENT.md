# Argon Project Management

## Current release line

**Minecraft 26.2 — Iris / Spooklementary optimization baseline**

Minecraft target: **26.2**

Current validation state is tracked in `docs/VALIDATION_STATUS.md`. At present, the project has CI build validation but no in-game validation.

## Version isolation

Argon isolates Minecraft versions at three levels.

### Git release lines

```text
main
└── mc/26.2
    └── dev/26.2/v0.1.0-iris-uniforms
```

`main` is the stable project integration line.

`mc/<minecraft-version>` is the long-lived maintenance line for one Minecraft version.

Feature work branches from the matching Minecraft line:

```text
dev/<minecraft-version>/<release>-<scope>
```

A future Minecraft 26.3 implementation branches into `mc/26.3`; it does not accumulate compatibility conditionals inside the 26.2 line.

### Source integration layers

Shared code remains outside a Minecraft-version package when it genuinely does not depend on version-specific Minecraft/Iris internals.

Version-sensitive Mixins and adapters live under an explicit package such as:

```text
io.github.nineteenreincarnation.argon.mixin.mc26_2
io.github.nineteenreincarnation.argon.version.mc26_2
```

A later port creates a sibling `mc26_3` layer instead of mutating the 26.2 layer into a multi-version conditional maze.

### Reference baselines

Version-specific external references remain under:

```text
References/<minecraft-version>/
```

Third-party source pointers are pinned per Minecraft version.

## Runtime compatibility gate

A version-specific Mixin fails closed when its external target version has not been validated.

For the current Iris work, the 26.2 integration is gated to the exact Fabric metadata baseline:

- Minecraft `26.2`
- Iris `1.11.4+mc26.2`

The human-facing Iris release is 1.11.4; its 26.2 Fabric build appends `+mc26.2` to the runtime version string.

Unknown Iris versions do not automatically receive the patch. Compatibility can be widened only after source review and runtime validation.

## Version format

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
- **Y — release version.** Y identifies the Argon release line within Minecraft 26.2. When development moves to the next release line, Y increments.
- **Z — development iteration.** Z increments by 1 for each development iteration made for the same Y release line.
- **+26.2 — Minecraft target.** Minecraft versions remain explicitly isolated from Argon's X/Y/Z numbering.

When Y increments, Z resets for the new release line.

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

Do not encode temporary validation states such as `dev`, `alpha`, `beta`, or `rc` into the version string by default. Validation status is tracked separately as compile/package/structure/runtime/visual/performance verification.

The repository may temporarily retain an older version string during migration. Do not choose the migration target implicitly; update it only after the project owner specifies the new X/Y/Z baseline.

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

1. Finish scoped issues for the current Y release line.
2. Run runtime correctness and compatibility checks.
3. Record benchmark evidence.
4. Finalize the changelog.
5. Merge the release work into `mc/26.2`.
6. Merge/tag the stable release as appropriate on `main`.
7. Tag the release commit using its full Argon version, for example `v1.0.12+26.2`.
8. Publish the CI-built JAR.
9. When starting the next release line, increment Y and reset Z before new development iterations begin.

X changes only by an explicit project-owner decision.
