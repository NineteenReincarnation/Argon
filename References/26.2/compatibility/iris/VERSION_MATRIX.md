# Iris Compatibility Matrix — Minecraft 26.2

Argon separates **Minecraft-version support**, **Iris structural compatibility**, and **runtime/performance validation**.

The primary benchmark baseline remains:

- Minecraft 26.2
- Iris 1.11.4
- Sodium 0.9.2
- Spooklementary 2.0.4

That baseline is not the only Iris patch release accepted by the current Phase 0 instrumentation.

## Published Fabric 26.2 family

| Iris release | Runtime version | Sodium line | Structural CI | In-game validation | Argon Phase 0 status |
| --- | --- | --- | --- | --- | --- |
| 1.11.0 | `1.11.0+mc26.2` | 0.9.0 | Passed | Not tested | Enabled |
| 1.11.1 | `1.11.1+mc26.2` | 0.9.0 | Passed | Not tested | Enabled |
| 1.11.2 | `1.11.2+mc26.2` | 0.9.1 | Passed | Not tested | Enabled |
| 1.11.4 | `1.11.4+mc26.2` | 0.9.2 | Passed | Not tested | Enabled / primary baseline |

No Fabric 26.2 release of Iris 1.11.3 is included in this matrix because the 1.11.3 Fabric release belongs to the 26.1 line.

## What structural CI checks

For every supported Iris JAR, CI resolves the actual published Fabric artifact and verifies the exact surface used by the current Argon Phase 0 Mixins:

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

CI also checks the Fabric runtime version embedded in each release JAR.

## Validation levels

**Structure verified**
: The published Iris JAR has the classes, fields, and method signatures required by Argon's current Mixin layer.

**Runtime verified**
: Minecraft has actually launched and the relevant Iris Mixins have applied successfully.

**Visual verified**
: Shader behavior has passed the relevant regression matrix.

**Performance verified**
: Reproducible benchmark data exists.

The current four Iris 26.2 Fabric releases are **structure verified only**. None has been claimed runtime verified yet.

## Runtime policy

Measurement-only Phase 0 instrumentation is enabled for every version in the structure-verified set.

An unknown Iris version does not disable Argon as a whole. It disables only Iris-specific Mixins until that version enters the compatibility matrix.

Behavior-changing optimizations may use a stricter compatibility set than instrumentation. Phase A will not automatically inherit every Phase 0-compatible version without its own validation decision.

## Policy for future Iris updates

Do not use a blanket `startsWith("1.11")` gate.

A new Iris 26.2 patch enters the candidate matrix first. CI checks the required target surface. If the structure is compatible, measurement-only instrumentation may be enabled. Runtime, visual, and performance validation remain separate states.
