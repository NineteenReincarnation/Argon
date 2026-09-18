# Argon References

This directory contains the development reference baseline for **Argon**, targeting **Minecraft Java Edition 26.2 on Fabric**.

The reference set is intentionally versioned. Official build/config examples are stored directly where their license allows it, while major third-party optimization and rendering projects are attached as pinned Git submodules so their complete source, history, and licenses remain attributable to the upstream projects.

## What is stored here

### `fabric-example-mod-26.2/`

A small snapshot of the official Fabric Example Mod's 26.2 branch, limited to build and metadata files we repeatedly consult while bootstrapping Argon.

The upstream template is CC0-1.0. Its upstream license is stored beside the snapshot.

The recorded 26.2 example baseline uses Minecraft 26.2, Java 25, Fabric Loader 0.19.5, Fabric Loom 1.17-SNAPSHOT, Fabric API 0.160.0+26.2, and Gradle 9.5.1.

### `third-party/`

Pinned complete source references for:

- Sodium
- Iris
- Lithium
- FerriteCore
- ImmediatelyFast
- EntityCulling

See `third-party/README.md` for exact commits, licenses, and clone instructions.

These source trees are used for compatibility and overlap analysis. Their presence does **not** authorize copying their implementation into Argon; each upstream license still applies.

## Shader baseline

See `SHADER_COMPATIBILITY.md`.

The primary shader regression target is **Spooklementary** on **Iris + Sodium** for Minecraft 26.2.

## Primary documentation

Use these sources before community tutorials:

- Fabric developer documentation: https://docs.fabricmc.net/develop/
- Development environment: https://docs.fabricmc.net/develop/getting-started/setting-up
- Project structure: https://docs.fabricmc.net/develop/getting-started/project-structure
- Fabric Loom: https://docs.fabricmc.net/develop/loom/
- Fabric Loader: https://docs.fabricmc.net/develop/loader/
- Fabric Mixin / Java bytecode guide: https://docs.fabricmc.net/develop/mixins/bytecode
- SpongePowered Mixin wiki: https://github.com/SpongePowered/Mixin/wiki
- SpongePowered Mixin source: https://github.com/SpongePowered/Mixin
- Oracle JDK Mission Control / JFR documentation: https://docs.oracle.com/en/java/java-components/jdk-mission-control/

See `STUDY_GUIDE.md` for the reading order.

## Minecraft source policy

Do not commit Mojang's Minecraft binaries or a full decompiled Minecraft source tree here. Fabric Loom prepares the Minecraft development environment locally.

## Profiling-data policy

Do not keep raw large JFR recordings, heap dumps, spark profiles, or benchmark captures in normal Git history. Keep methodology, compact representative results, profiler conclusions, and reproduction instructions.

## Updating this directory

When the baseline changes:

1. record the exact Minecraft version;
2. pin the exact upstream revision;
3. verify the upstream license;
4. update source submodules intentionally rather than tracking moving branches;
5. update compatibility test baselines when the user's normal render stack changes.

Baseline recorded: **2026-09-18**.
