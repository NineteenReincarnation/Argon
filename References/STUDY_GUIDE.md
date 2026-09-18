# Argon Development Study Guide

This is the minimum technical reading path before invasive optimization work is merged into Argon.

## 1. Fabric 26.2 project model

Primary sources:

- https://docs.fabricmc.net/develop/getting-started/setting-up
- https://docs.fabricmc.net/develop/getting-started/project-structure
- https://docs.fabricmc.net/develop/loom/
- https://docs.fabricmc.net/develop/loader/

Understand what Fabric Loader, Fabric API and Loom each own; common versus client source sets; `fabric.mod.json`, entrypoints, dependencies and Mixin configs; and the Java 25 baseline.

The local `fabric-example-mod-26.2/` snapshot is the canonical small build/config reference.

## 2. Mixin and bytecode

Primary sources:

- https://github.com/SpongePowered/Mixin/wiki
- https://github.com/SpongePowered/Mixin
- https://docs.fabricmc.net/develop/mixins/bytecode

Before non-trivial patches, understand target descriptors, `@Inject`, injection points, cancellation, `@Shadow`, redirects/expression modification, local-capture risks, Mixin priority, transform interaction, and the bytecode consequences of source-level edits.

Argon rule: use the least invasive injection that can implement a measured optimization safely.

## 3. Java 25 performance fundamentals

Primary source:

- https://docs.oracle.com/en/java/java-components/jdk-mission-control/

Study allocation rate versus retained heap, young-generation GC, object lifetime, CPU sampling, lock contention, thread scheduling, JFR events, warm-up, and benchmark noise.

Do not infer performance from source appearance alone.

## 4. Minecraft performance investigation

For every candidate optimization:

1. reproduce the workload;
2. profile vanilla Minecraft 26.2;
3. identify the actual hot path;
4. inspect the relevant vanilla implementation;
5. run the same workload with the reference optimization stack;
6. inspect upstream mods that already touch that subsystem;
7. design the smallest patch that still has measurable value;
8. test correctness and compatibility;
9. benchmark again.

Important metrics include frame-time distribution, 1% / 0.1% lows, MSPT, allocation rate, GC pauses, target-path CPU time, and retained memory over time.

Average FPS alone is not a sufficient benchmark.

## 5. Optimization ecosystem

Read `OPTIMIZATION_ECOSYSTEM.md` before touching a subsystem owned by Sodium, Lithium, FerriteCore, ImmediatelyFast, or EntityCulling.

## 6. Design discipline

- keep strongly related logic local;
- do not create files or interfaces just to satisfy a pattern;
- extract stable interfaces where multiple modules genuinely need the same capability;
- keep compatibility decisions centralized;
- give each optimization a stable ID;
- make patches independently disableable where practical;
- document cache lifetime, invalidation and thread ownership;
- avoid hidden global mutable state;
- separate pure calculations from Minecraft state access when that improves testing or benchmarking.

Abstraction should reduce future coupling, not increase present-day navigation cost.
