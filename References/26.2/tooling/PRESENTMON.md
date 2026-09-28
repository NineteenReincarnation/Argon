# PresentMon tooling reference — Argon Probe / Minecraft 26.2

## Purpose

This record pins the upstream source state used while designing and statically auditing Argon Probe P0's PresentMon integration.

It is a research/compatibility reference. PresentMon is **not vendored** into Argon and is not a dependency of the Argon Fabric mod.

## Upstream

- Project: `GameTechDev/PresentMon`
- Upstream: https://github.com/GameTechDev/PresentMon
- Audited branch: `main`
- Audited commit: `bd2908fe9d2b277be4e1e3bdc0bda239d2fe68c9`
- License: MIT
- Current Argon Probe validation target: Windows + Minecraft Java 26.2 / Fabric

The upstream commit is pinned so future source/CLI changes can be compared against a known audit point. Probe does **not** decide compatibility by matching this commit or by checking a version prefix.

## P0 CLI surface audited

Argon Probe P0 currently needs the official PresentMon console CLI to expose:

```text
--process_id
--output_stdout
--no_console_stats
--qpc_time
--v2_metrics
--no_track_input
--no_track_gpu
--timed
--terminate_after_timed
--terminate_on_proc_exit
--session_name
```

When the optional `--track_etw_status` surface is available, Probe enables it and consumes PresentMon's ETW quality counters. Absence of this optional surface does not abort capture, but the resulting report is quality-degraded and must not be treated as benchmark-quality evidence.

Probe performs a runtime preflight against `PresentMon --help` and refuses the P0 capture with a concrete missing-option error if this surface is not available.

P0 uses `--no_track_gpu` by default so the base capture requests CPU/display evidence without unconditional GPU-duration tracking. The Probe CLI flag `--track-gpu` removes that suppression for a targeted capture. The actual overhead difference remains a runtime benchmark question; the default is chosen to minimize requested tracing work, not to claim a measured percentage improvement.

This is intentionally capability/surface-based rather than a rule such as:

```text
version.startsWith(...)
```

## P0 CSV surface audited

The v2/QPC parser requires:

```text
ProcessID
SwapChainAddress
CPUStartQPC
FrameTime
```

The following are optional and are recorded only when present:

```text
CPUBusy
CPUWait
GPUTime
GPUBusy
DisplayedTime
PresentMode
PresentRuntime
EtwEventsLost
EtwBuffersLost
OverflowedPresents
```

The three ETW status fields are emitted by PresentMon when `--track_etw_status` is enabled. Probe records their maximum observed values across the capture. Non-zero loss/overflow values degrade capture quality. If the status capability is unavailable, capture may continue but cannot receive a `GOOD` quality classification.

Missing optional metrics must disable only the corresponding evidence. They must not fabricate zero values or make unrelated Probe collection fail.

## Metric semantics

At the audited upstream source state:

- `CPUStartQPC` is the CPU frame-start time represented as a `QueryPerformanceCounter()` value when `--qpc_time` is used.
- v2 `FrameTime` is a **CPU frame-time** metric: time from the start of one CPU frame until the CPU starts the next frame.
- `DisplayedTime` is how long a displayed frame remained on screen and is `NA` when that frame was not displayed.
- `FrameTime` and `DisplayedTime` are not interchangeable.

PresentMon also documents that applications reported with runtime `Other`—typical for OpenGL or Vulkan—have less presentation instrumentation, and CPU frame-time-derived values may be slightly less accurate.

Minecraft Java's target renderer is therefore a runtime-validation requirement. Argon Probe must not turn a structure-correct PresentMon row into a claim that its metric semantics have already been validated for Minecraft/OpenGL.

## Current Argon Probe status

Verified statically / by CI:

- required v2/QPC header parsing;
- official-style v2 fixture parsing;
- QPC raw-value handling;
- separation of CPU `FrameTime` from display-side `DisplayedTime`;
- distinction between an unavailable `DisplayedTime` column and a per-row `NA`;
- required CLI-option preflight logic;
- optional `--track_etw_status` capability detection;
- ETW events-lost / buffers-lost / overflowed-present CSV parsing;
- quality downgrade rules for missing or non-zero ETW loss evidence.

Not yet verified:

- real Minecraft 26.2/OpenGL PresentMon capture;
- which PresentMon frame metric(s) should be primary for final Argon benchmark conclusions;
- real Minecraft validation of ETW loss reporting semantics;
- Probe OFF/ON measurement overhead.

Those remain runtime gates in issue #9.

## Update rule

When PresentMon changes:

1. inspect the new CLI/CSV/API surface;
2. compare it with this pinned audit point;
3. update parser/preflight logic only if the actual surface requires it;
4. run Probe CI;
5. perform Minecraft runtime validation before widening any runtime-verified claim;
6. update this reference with the new audited commit.

Do not copy large parts of PresentMon source into Argon merely because the source is available.
