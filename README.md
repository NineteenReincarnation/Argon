# Argon

Argon is a modular performance optimization mod for **Minecraft Java Edition 26.2**, designed to complement rather than replace the established optimization ecosystem.

## Current development line

**v0.1.0 — Iris / Spooklementary**

Current development branch:

```text
dev/26.2/v0.1.0-iris-uniforms
```

The first measured target is the **Iris custom-uniform pipeline**, using **Spooklementary 2.0.4** on **Iris 1.11.4 + Sodium 0.9.2** as the primary planned real-world shader workload.

### Validation status

The project currently has **CI compile validation only**. No in-game test has been performed yet, and no performance improvement is claimed yet.

See [docs/VALIDATION_STATUS.md](docs/VALIDATION_STATUS.md).

## Version isolation

Minecraft-version-specific work is isolated in dedicated release branches and source integration packages.

```text
main
└── mc/26.2
    └── dev/26.2/v0.1.0-iris-uniforms
```

Version-sensitive integration code uses explicit packages such as `mc26_2`; reference sources use `References/26.2/`.

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

## References

Version-specific source and compatibility baselines live under `References/<minecraft-version>/`.

The Minecraft 26.2 baseline includes pinned source for Sodium, Iris, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, and Spooklementary 2.0.4.

## License

A project license has not yet been selected.
