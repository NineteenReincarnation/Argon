# Development Runtime — Minecraft 26.2

This document describes the reproducible development client used for the primary Argon shader baseline.

## Primary shader stack

- Minecraft 26.2
- Fabric Loader 0.19.5
- Fabric API 0.160.0+26.2
- Sodium 0.9.2
- Iris 1.11.4
- Spooklementary 2.0.4
- Argon current v0.1.0 development build

The Sodium/Iris versions are development runtime dependencies only. They are **not bundled into the Argon JAR**.

## First setup

Initialize the pinned reference submodules:

```bash
git submodule update --init --recursive
```

The Spooklementary source must exist at:

```text
References/26.2/third-party/spooklementary/
```

## Start the primary shader development client

Linux/macOS:

```bash
./gradlew runClient -Pargon_dev_shader_stack=true
```

Windows PowerShell:

```powershell
.\gradlew.bat runClient -Pargon_dev_shader_stack=true
```

When this property is enabled, Gradle:

1. adds the pinned Fabric API, Sodium, and Iris releases to the local development runtime;
2. copies the pinned Spooklementary 2.0.4 source tree into the development instance's `run/shaderpacks/`;
3. writes `run/config/iris.properties` selecting that shader pack and enabling shaders;
4. starts the normal Loom development client with Argon loaded.

The generated development files under `run/` are ignored by Git.

## Normal development client

To run Minecraft without the primary Iris/Sodium shader stack:

```bash
./gradlew runClient
```

This is useful for checking that Argon itself does not require Iris.

## Important validation distinction

A successful Gradle launch configuration or dependency resolution is not an in-game validation result.

Runtime validation begins only after the client actually reaches the intended test state and the logs confirm the relevant Argon Mixins are active.

## Testing other structure-verified Iris patches

The Phase 0 compatibility matrix currently structure-verifies Iris 1.11.0, 1.11.1, 1.11.2, and 1.11.4 for Minecraft 26.2.

The automated development stack intentionally remains pinned to the **primary baseline** (1.11.4 + Sodium 0.9.2). Older Iris patches can be tested separately when runtime compatibility testing begins.


## One-command Windows Phase 0 run

For the first runtime validation, use:

```powershell
.\scripts\run-phase0.ps1
```

It launches the primary baseline and extracts Argon's relevant log lines after the client exits. See `docs/FIRST_RUNTIME_TEST.md`.


## JFR

Add `-Jfr` to the Windows helper to start a Java Flight Recorder capture. For optimization A/B work, pair it with `-PerformanceMode` so the detailed Phase 0 simulator does not walk the uniform map during the performance capture.


## Iris patch selector

`scripts/run-phase0.ps1` accepts `-IrisVersion 1.11.0|1.11.1|1.11.2|1.11.4`. The script supplies the matching published Modrinth Iris/Sodium artifact pair to Gradle for that run. The repository defaults remain pinned to the 1.11.4 / 0.9.2 primary baseline.
