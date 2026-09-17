# Argon

Argon is a modular performance optimization mod for **Minecraft Java Edition 26.2**, built with compatibility and measurable performance improvements as first-class goals.

The project is intended to complement the existing Minecraft optimization ecosystem rather than replace it. Argon focuses on performance gaps that remain after established optimization mods have done their work, while avoiding unnecessary overlap with their core responsibilities.

## Project goals

Argon is designed around four primary goals:

- **Improve frame-time stability** — reduce stutter, frame-time spikes, and low-percentile performance issues rather than optimizing only average FPS.
- **Reduce CPU and allocation overhead** — identify hot paths, unnecessary temporary allocations, redundant computation, and avoidable garbage-collection pressure.
- **Preserve vanilla behavior** — normal optimization patches should not intentionally alter game mechanics, simulation results, networking behavior, or gameplay rules.
- **Remain compatible with the optimization ecosystem** — avoid duplicating or conflicting with optimizations already provided by established mods.

## Target

- **Minecraft:** 26.2
- **Edition:** Java Edition
- **Initial mod loader:** Fabric
- **Status:** Early development

Minecraft 26.2 is the current development target for Argon. Support for other Minecraft versions or additional mod loaders is not part of the initial development scope.

## Optimization scope

Argon will investigate performance improvements in areas such as:

- temporary object allocation and garbage-collection pressure;
- repeated calculations and cacheable lookups;
- client-side non-rendering overhead;
- entity and world hot paths not already covered effectively elsewhere;
- memory-access patterns and hot-path data structures;
- scheduling and main-thread workload distribution;
- frame-time consistency and micro-stutter.

Optimization targets are selected through profiling and benchmarking. A subsystem is not modified simply because it appears expensive: existing implementations and other optimization mods are checked first.

## Compatibility philosophy

Argon is intended to work alongside common optimization mods, including:

- Sodium
- Lithium
- FerriteCore
- ImmediatelyFast
- EntityCulling

Where another mod already owns or heavily rewrites a subsystem, Argon should avoid competing with it unless an optimization is demonstrably safe and complementary.

Argon will eventually include a compatibility layer capable of detecting installed optimization mods and disabling individual patches when overlap or incompatibility is known.

In practical terms, the intended stack is:

```text
Sodium
+ Lithium
+ FerriteCore
+ ImmediatelyFast
+ EntityCulling
+ Argon
```

Argon is not intended to be a replacement for that stack. Its role is to optimize the remaining performance gaps.

## Optimization rules

A normal Argon optimization should satisfy the following principles:

1. The performance problem must be observable through profiling or a reproducible benchmark.
2. The optimization must provide a measurable improvement in its target workload.
3. Vanilla behavior should remain unchanged unless the feature is explicitly marked as experimental or aggressive.
4. The patch should be independently configurable whenever practical.
5. Compatibility with major optimization mods must be considered before the patch is enabled by default.
6. A patch that produces negligible benefit while significantly increasing complexity or compatibility risk should not be merged.

Argon will prioritize metrics such as frame time, 1%/0.1% lows, allocation rate, garbage-collection pauses, and MSPT in addition to average FPS.

## Planned architecture

The project is expected to evolve around independently controllable optimization modules:

```text
argon
├── core
│   ├── bootstrap
│   ├── compatibility
│   ├── config
│   └── diagnostics
├── optimization
│   ├── allocation
│   ├── cache
│   ├── client
│   ├── entity
│   ├── memory
│   ├── scheduling
│   └── world
├── compat
└── benchmark
```

The exact structure may change as the first profiling and implementation work is completed.

## Development approach

Argon follows a profile-first workflow:

```text
Profile
  ↓
Identify a hot path
  ↓
Check vanilla implementation and existing optimization mods
  ↓
Design the smallest safe optimization
  ↓
Benchmark
  ↓
Test vanilla behavior
  ↓
Test compatibility
  ↓
Merge
```

The project will establish its benchmarking and compatibility infrastructure before attempting large or invasive optimizations.

## Current stage

Argon is currently in **Phase 0**.

The immediate development goals are:

- establish the Minecraft 26.2 Fabric project;
- create configuration and diagnostics foundations;
- build compatibility detection infrastructure;
- establish reproducible benchmark and profiling workflows;
- profile both vanilla Minecraft 26.2 and a representative optimization-mod stack;
- select the first optimization target from measured data.

## License

A project license has not yet been selected.
