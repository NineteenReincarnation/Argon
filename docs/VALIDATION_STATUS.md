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
- Iris 1.11.0, 1.11.1, 1.11.2, and 1.11.4 Fabric builds for Minecraft 26.2 all pass the current Phase 0 Mixin target-surface audit and the static Phase A behavior-assumption audit.

### Compatibility meaning

Those four Iris releases are **structure verified**, not runtime verified.

The current Phase 0 instrumentation may load for a structure-verified Iris version. An unknown Iris version disables only Iris-specific Argon Mixins; the rest of Argon remains loadable.

The primary benchmark baseline remains Iris 1.11.4 + Sodium 0.9.2.

### Not yet verified in-game

No local Minecraft game test has been performed yet for the current Argon development build.

Therefore the project does **not** currently claim that:

- Fabric Loader reaches the main menu/world with Argon installed;
- any of the structure-verified Iris versions is runtime verified;
- the Iris instrumentation Mixins apply successfully in a live game;
- Iris + Sodium + Spooklementary renders correctly with Argon;
- shader reload, dimension changes, or profile changes invalidate state correctly at runtime;
- the instrumentation has acceptable measurement overhead;
- any Argon optimization improves FPS, frame time, CPU usage, allocation rate, or GPU/driver cost.

These remain open validation work.

## Claim rule

Project documentation, issues, pull requests, and changelogs must distinguish:

- **compile verified**
- **package verified**
- **structure verified**
- **runtime verified**
- **visual regression verified**
- **performance benchmark verified**

A successful compile/package/structure audit is never treated as evidence of runtime correctness or performance improvement.


### Experimental Phase A compatibility

The default-off Phase A uniform deduplication switch may be explicitly enabled on Iris 1.11.0, 1.11.1, 1.11.2, or 1.11.4 for Minecraft 26.2 because all four pass the current structural and bytecode behavior audits.

This does not promote any of them to runtime-verified status.
