# Argon

Argon is a modular performance optimization mod for **Minecraft Java Edition 26.2**, designed to complement rather than replace the established optimization ecosystem.

## Current development line

**Minecraft 26.2 — Iris / Spooklementary development line**

Active development is committed directly to `main`. Historical development branches may remain in Git history, but they are not part of the current workflow.

The first measured target is the **Iris custom-uniform pipeline**, using **Spooklementary 2.0.4** on **Iris 1.11.4 + Sodium 0.9.2** as the primary planned real-world shader workload.

### Validation status

The project currently has **compile, package, and structure validation** through CI. No in-game, visual-regression, or performance-benchmark validation has been performed yet, and no performance improvement is claimed yet.

See [docs/VALIDATION_STATUS.md](docs/VALIDATION_STATUS.md).

## Version management

Argon develops directly on `main` and uses the project version number to track release progression.

Current version format:

```text
X.Y.Z+<minecraft-version>
```

For the current line, `X` is the major content version, `Y` is the release line, and `Z` is the development iteration. Argon starts from the `1.1` release line; there is no `1.0` line. The Minecraft suffix identifies the target game version and does not replace Argon's own release numbering.

Version-sensitive compatibility code may still keep explicit boundaries where required by upstream API or bytecode differences.

## Goals

- improve frame-time stability and low-percentile performance;
- reduce avoidable CPU work, allocation pressure, and driver calls;
- preserve vanilla/game and shader-visible behavior for normal optimizations;
- coexist with Sodium, Iris, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, and other established optimization mods;
- require profiling/benchmark evidence before invasive optimization work is merged.

## Build

Requirements: **JDK 25** and Git.

```bash
git clone --recurse-submodules https://github.com/NineteenReincarnation/Argon.git
cd Argon
./gradlew build
```

Windows PowerShell:

```powershell
.\gradlew.bat build
```

The installable JAR is produced under `build/libs/`.

GitHub Actions builds development JAR artifacts automatically.

For the pinned Iris + Sodium + Spooklementary development client:

```powershell
.\\gradlew.bat runClient -Pargon_dev_shader_stack=true
```

See [docs/DEVELOPMENT_RUNTIME.md](docs/DEVELOPMENT_RUNTIME.md).

## Development model

```text
Reproduce
  ↓
Profile
  ↓
Check Minecraft + reference optimization sources
  ↓
Implement the smallest safe patch
  ↓
Benchmark
  ↓
Regression / compatibility test
  ↓
Merge
```

See [docs/PROJECT_MANAGEMENT.md](docs/PROJECT_MANAGEMENT.md).

## Development tools

Argon Probe is a standalone performance-evidence collector under active development. It is intentionally separate from the Fabric mod. See [tools/argon-probe/](tools/argon-probe/README.md).

## References

Version-specific source and compatibility baselines live under `References/<minecraft-version>/`.

The Minecraft 26.2 baseline includes pinned source for Sodium, Iris, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, and Spooklementary 2.0.4.

## License

A project license has not yet been selected.
