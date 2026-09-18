# Argon

Argon is a modular performance optimization mod for **Minecraft Java Edition 26.2**, designed to complement rather than replace the established optimization ecosystem.

## Current development line

**v0.1.0 — Iris / Spooklementary**

The first measured target is the **Iris custom-uniform pipeline**, using **Spooklementary 2.0.4** on **Iris 1.11.4 + Sodium 0.9.2** as the primary real-world shader workload.

The initial Phase 0 build performs instrumentation only. It does not intentionally change shader output.

## Goals

- improve frame-time stability and low-percentile performance;
- reduce avoidable CPU work, allocation pressure, and driver calls;
- preserve vanilla/game and shader-visible behavior for normal optimizations;
- coexist with Sodium, Iris, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, and other established optimization mods;
- require profiling/benchmark evidence before invasive optimization work is merged.

## Build

Requirements: **JDK 25** and Git.

Clone:

```bash
git clone --recurse-submodules https://github.com/NineteenReincarnation/Argon.git
cd Argon
```

Build:

```bash
./gradlew build
```

Windows PowerShell:

```powershell
.\gradlew.bat build
```

The installable JAR is produced under `build/libs/`.

Run the Fabric development client with `./gradlew runClient`.

GitHub Actions also builds development JAR artifacts automatically for `main`, `dev/**`, fix branches, and pull requests.

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

Versioning, branch policy, merge gates, and the v0.1.0 phases are in [docs/PROJECT_MANAGEMENT.md](docs/PROJECT_MANAGEMENT.md).

## References

Version-specific source and compatibility baselines live under `References/<minecraft-version>/`.

The Minecraft 26.2 baseline includes pinned source for Sodium, Iris, Lithium, FerriteCore, ImmediatelyFast, EntityCulling, and Spooklementary 2.0.4.

## License

A project license has not yet been selected.
