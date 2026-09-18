# Validation Status

This file distinguishes what Argon has actually validated from what is only designed or compiled.

## Current status — v0.1.0-dev / Minecraft 26.2

### Verified by CI

- Gradle project configuration builds successfully with Java 25.
- Minecraft 26.2 Fabric development sources resolve and compile.
- Phase 0 Iris instrumentation source compiles.
- The remapped installable JAR is produced successfully.
- CI inspects the packaged JAR and confirms the expected core classes, 26.2 Mixin plugin, Iris Mixin, `fabric.mod.json`, and Mixin configuration are present.
- The development JAR is uploaded as a GitHub Actions artifact.

### Not yet verified in-game

No local Minecraft game test has been performed yet for the current Argon development build.

Therefore the project does **not** currently claim that:

- Fabric Loader reaches the main menu/world with Argon installed;
- the 26.2 Mixin compatibility gate behaves correctly at runtime;
- the Iris instrumentation Mixins apply successfully;
- Iris + Sodium + Spooklementary renders correctly with Argon;
- shader reload, dimension changes, or profile changes invalidate state correctly at runtime;
- the instrumentation has acceptable measurement overhead;
- any Argon optimization improves FPS, frame time, CPU usage, allocation rate, or GPU/driver cost.

These remain open validation work.

## Claim rule

Project documentation, issues, pull requests, and changelogs must distinguish:

- **compile verified**
- **package verified**
- **runtime verified**
- **visual regression verified**
- **performance benchmark verified**

A successful compile/package is never treated as evidence of runtime correctness or performance improvement.
