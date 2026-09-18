# Phase B — Dependency-aware Iris Uniform Evaluation

Status: **experimental, default OFF**

Minecraft line: **26.2**

Initial runtime baseline: **Iris 1.11.4 + Sodium 0.9.2 + Spooklementary 2.0.4**

## Goal

Reduce repeated custom-uniform expression evaluation without changing shader-visible values.

## Safety model

Phase B only considers a custom expression cacheable when its resolved Stareval expression is classified as **PURE**.

The classifier uses Iris' resolved expression graph and function metadata:

- `TypedFunction.isPure() == false` -> stateful, never skipped;
- `random` / `randomInt` -> nondeterministic, never skipped;
- unknown dynamic functions -> conservative UNKNOWN, never skipped;
- constant/variable nodes and recognized pure static functions -> pure candidates when all child expressions are pure.

This specifically excludes Spooklementary's `smooth(...)` chains.

## Dependency revision rule

Iris already builds a direct dependency graph for custom uniforms.

Argon snapshots the revisions of a pure candidate's direct dependencies after a real evaluation.

On the next `CachedUniform.update()`:

```text
all dependency revisions unchanged
    -> skip evaluation

any dependency revision changed
    -> evaluate normally and refresh snapshot
```

A candidate with no dependencies evaluates once and may then remain cached.

## Failure behavior

Any unexpected Iris/Stareval reflection shape or dependency without Argon's revision state disables Phase B for the session and returns to Iris' original evaluation behavior.

Phase B does not alter expression ASTs or shader-pack source.

## Current validation level

The code is compile/package validated only.

Iris 1.11.4 is initially the only Phase B runtime gate. CI audits the expression-analysis surface on all published Iris 1.11.x builds for Minecraft 26.2; the gate can be widened only after those checks pass and the implementation is reviewed.

No runtime, visual, or performance claim is made yet.

## Development switch

```text
-Dargon.experimental.irisUniformEvaluation=true
```

Repository helper:

```powershell
.\scripts\run-phase0.ps1 -EnablePhaseB
```

Phase A and Phase B can be enabled together for experimental testing:

```powershell
.\scripts\run-phase0.ps1 -EnablePhaseA -EnablePhaseB
```
