# Validation Status

This file distinguishes what Argon has actually validated from what is only designed or compiled.

## Current status — v0.1.0-dev / Minecraft 26.2

### Verified

- Gradle project configuration builds successfully in GitHub Actions with Java 25.
- Minecraft 26.2 Fabric development sources resolve and compile.
- Phase 0 Iris instrumentation source compiles in CI.

### Not yet verified in-game

No local Minecraft game test has been performed yet for the current Argon development build.

Therefore the project does **not** currently claim that:

- the Iris instrumentation Mixins apply successfully at runtime;
- Minecraft reaches a world with Argon installed;
- Iris + Sodium + Spooklementary renders correctly with Argon;
- the instrumentation has acceptable measurement overhead;
- any Argon optimization improves FPS, frame time, CPU usage, allocation rate, or GPU/driver cost.

These remain open validation work.

## Claim rule

Project documentation, issues, pull requests, and changelogs must distinguish:

- **CI build verified**
- **runtime verified**
- **visual regression verified**
- **performance benchmark verified**

A successful compile is never treated as evidence of runtime correctness or performance improvement.
