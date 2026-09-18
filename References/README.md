# Argon References

This directory contains the development reference baseline for **Argon**, targeting **Minecraft Java Edition 26.2 on Fabric**.

The goal is to keep the reference set small, authoritative, versioned, and safe to keep in the Argon repository. It is not a vendor directory for every optimization mod.

## What is stored here

### `fabric-example-mod-26.2/`

A small snapshot of the official Fabric Example Mod's 26.2 branch, limited to build and metadata files we will repeatedly consult while bootstrapping Argon.

The upstream template is CC0-1.0. Its upstream license is stored beside the snapshot and applies to those copied template files.

Upstream:
https://github.com/FabricMC/fabric-example-mod/tree/26.2

At the time this reference baseline was recorded, the official 26.2 example uses:

- Minecraft 26.2
- Java 25
- Fabric Loader 0.19.5
- Fabric Loom 1.17-SNAPSHOT
- Fabric API 0.160.0+26.2
- Gradle 9.5.1

These are a **reference baseline**, not a promise that Argon will permanently pin every dependency to these exact patch versions.

## Primary documentation

Use these sources before community tutorials:

- Fabric developer documentation (26.2): https://docs.fabricmc.net/develop/
- Development environment: https://docs.fabricmc.net/develop/getting-started/setting-up
- Project structure: https://docs.fabricmc.net/develop/getting-started/project-structure
- Fabric Loom: https://docs.fabricmc.net/develop/loom/
- Fabric Loader: https://docs.fabricmc.net/develop/loader/
- Fabric Mixin / Java bytecode guide: https://docs.fabricmc.net/develop/mixins/bytecode
- SpongePowered Mixin wiki: https://github.com/SpongePowered/Mixin/wiki
- SpongePowered Mixin source: https://github.com/SpongePowered/Mixin
- Oracle JDK Mission Control / JFR documentation: https://docs.oracle.com/en/java/java-components/jdk-mission-control/

See `STUDY_GUIDE.md` for the order in which these materials should be used.

## Optimization-mod reference baseline

See `OPTIMIZATION_ECOSYSTEM.md`.

These projects are used to answer three questions:

1. Is the hotspot already optimized elsewhere?
2. Will an Argon patch overlap or conflict with an established implementation?
3. Is the proposed optimization still useful when the common optimization stack is installed?

Their source trees are intentionally **not copied into this repository**. Public source does not mean code may be copied freely, and several projects use licenses with different obligations or restrictions. Read the pinned upstream revision directly when implementing or reviewing a patch.

## Minecraft source policy

Do not commit Mojang's Minecraft binaries or a full decompiled Minecraft source tree here.

Fabric Loom is responsible for preparing the Minecraft development environment locally. When a specific vanilla class is central to a design decision, document the class/method name and Minecraft version in Argon documentation instead of vendoring the entire game source.

## Profiling-data policy

Do not keep raw multi-megabyte or gigabyte JFR recordings, heap dumps, spark profiles, or benchmark captures in this directory.

Keep benchmark methodology, compact representative results, relevant profiler conclusions, and reproduction instructions. Large raw captures belong outside normal Git history.

## Updating this directory

When the baseline changes:

1. record the exact Minecraft version;
2. record the exact upstream tag/branch/commit where practical;
3. verify the upstream license before copying any file;
4. prefer links plus Argon-authored notes over vendoring third-party source;
5. keep old data only when it remains useful for regression or migration work.

Baseline recorded: **2026-09-18**.
