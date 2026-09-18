# Iris Compatibility Matrix — Minecraft 26.2

Argon separates **Minecraft-version support**, **Iris structural compatibility**, and **runtime/performance validation**.

The primary benchmark baseline remains:

- Minecraft 26.2
- Iris 1.11.4
- Sodium 0.9.2
- Spooklementary 2.0.4

That baseline is not intended to be the only usable Iris patch release.

## Published Fabric 26.2 family

| Iris release | Runtime version | Sodium line | Structural CI | In-game validation | Argon status |
| --- | --- | --- | --- | --- | --- |
| 1.11.0 | `1.11.0+mc26.2` | 0.9.0 | Pending | Not tested | Candidate |
| 1.11.1 | `1.11.1+mc26.2` | 0.9.0 | Pending | Not tested | Candidate |
| 1.11.2 | `1.11.2+mc26.2` | 0.9.1 | Pending | Not tested | Candidate |
| 1.11.4 | `1.11.4+mc26.2` | 0.9.2 | Pending | Not tested | Primary baseline |

No Fabric 26.2 release of Iris 1.11.3 is included in this matrix because the 1.11.3 Fabric release was for the 26.1 line, not the 26.2 line.

## What structural CI checks

For every candidate Iris JAR, CI resolves the actual published Fabric artifact and verifies the exact surface used by the current Argon Phase 0 Mixins:

- `IrisRenderingPipeline.beginLevelRendering()`
- `CustomUniforms.locationMap`
- `CustomUniforms.update()`
- `CustomUniforms.push(Object)`
- `CustomUniforms.optimise()`
- `CustomUniforms.mapholderToPass(...)`
- `CachedUniform.changed`
- `CachedUniform.update()`
- `CachedUniform.pushIfChanged(int)`
- `doUpdate()` on the supported cached-uniform subclasses

It also checks the Fabric runtime version embedded in the release JAR.

## Validation levels

**Candidate**
: Published for Minecraft 26.2, but the current Argon target surface has not yet passed CI.

**Structure verified**
: The published Iris JAR has the classes/fields/method signatures required by Argon's Mixin layer. This is sufficient to consider enabling measurement-only Phase 0 instrumentation, but it is not proof of runtime correctness.

**Runtime verified**
: Minecraft has actually launched and the relevant Iris Mixins have applied successfully.

**Visual verified**
: Shader behavior has passed the relevant regression matrix.

**Performance verified**
: Reproducible benchmark data exists.

A version can be structurally compatible while still being runtime-unverified.

## Policy for future Iris updates

Do not use a blanket `startsWith("1.11")` gate.

A new Iris 26.2 patch enters the candidate matrix first. CI then checks the required target surface. Behavior-changing Argon optimizations may use a stricter compatibility set than measurement-only instrumentation.
