# Argon References — Minecraft 26.2

This directory is the complete development reference baseline for **Argon on Minecraft Java Edition 26.2 / Fabric**.

## Baseline

- Minecraft 26.2
- Java 25
- Fabric Loader 0.19.5 reference baseline
- Fabric Loom 1.17-SNAPSHOT reference baseline
- Fabric API 0.160.0+26.2 reference baseline
- Gradle 9.5.1 reference baseline

## Contents

- `fabric-example-mod/` — compact official Fabric 26.2 project/config snapshot
- `third-party/` — pinned complete source trees for the optimization/rendering ecosystem
- `OPTIMIZATION_ECOSYSTEM.md` — overlap and compatibility ownership map
- `SHADER_COMPATIBILITY.md` — Iris + Sodium + Spooklementary regression baseline
- `PROFILING.md` — profiling/benchmark rules
- `STUDY_GUIDE.md` — prerequisite reading path

## Minecraft source policy

Do not vendor Mojang Minecraft binaries or a full decompiled Minecraft source tree here. Loom prepares the local development source environment.

## Raw profiling data

Do not keep large JFR recordings, heap dumps, spark captures, or other bulky generated data in normal Git history. Keep the methodology, compact representative results, conclusions, and reproduction instructions.
