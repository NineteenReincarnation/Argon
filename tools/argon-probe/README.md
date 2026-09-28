# Argon Probe — Performance Evidence Collection Plan

> Status: design baseline  
> Scope: independent developer tooling for Argon; **not part of the Argon Fabric mod**  
> Initial validation target: Windows + Minecraft Java 26.2 / Fabric  
> Repository location: `tools/argon-probe/`

## 1. Purpose

Argon Probe is an independent, low-interference performance evidence collector for Argon development.

Its job is not to become another general-purpose profiler, FPS overlay, benchmark suite, or automatic “AI root-cause detector”. Its job is to remove the repetitive manual work currently required before an Argon optimization can be investigated:

- identifying the correct Minecraft process;
- collecting frame-time data;
- collecting JVM evidence;
- collecting low-frequency system telemetry;
- preserving evidence around stutters and other incidents;
- recording the exact runtime environment;
- checking whether two benchmark runs are actually comparable;
- packaging the evidence into one report that can be reviewed later.

The intended workflow is:

```text
Run Minecraft normally
        ↓
Argon Probe records low-cost evidence
        ↓
Exit Minecraft
        ↓
argon-report-*.zip
        ↓
Review report / identify missing evidence
        ↓
Optional targeted second capture
        ↓
Read Minecraft / Iris / Sodium / other source
        ↓
Implement smallest safe optimization
        ↓
Benchmark separately
```

Argon Probe exists to support Argon's measurement-driven workflow. It must never turn “we collected a trace” into “we proved a performance improvement”.

---

## 2. Non-goals

The first versions of Argon Probe explicitly do **not** aim to:

- ship inside the Argon mod JAR;
- inject Mixins into Minecraft;
- add permanent instrumentation to Argon runtime code;
- replace JFR, PresentMon, WPR, Perfetto, spark, or other mature profiling systems;
- upload data to a cloud service;
- provide a web dashboard;
- provide an in-game overlay;
- continuously perform heavy allocation tracing;
- continuously instrument Java methods;
- infer a definitive root cause from a few utilization percentages;
- hard-code current Argon Phase A / Phase B / Iris uniform logic into the core;
- require every future Argon optimization to modify Probe core code;
- make performance claims from a single run.

If one of these becomes a real requirement later, it must be justified by measured need rather than added speculatively.

---

## 3. Core design principles

### 3.1 Evidence before interpretation

Probe records facts such as:

- frame-time anomaly occurred;
- GPU was or was not saturated;
- a GC pause overlapped the incident;
- render-thread samples are available;
- allocation rate increased;
- process CPU changed;
- telemetry quality degraded.

It should avoid producing conclusions such as:

```text
Root cause: CPU bottleneck
Confidence: 82%
```

unless a future analysis layer has a separately validated methodology.

The primary artifact is evidence. Interpretation can be regenerated later.

### 3.2 Generic core, Argon-specific profiles

Probe Core must not contain feature-specific names such as:

```text
PhaseACollector
PhaseBCollector
IrisUniformAnalyzer
SpooklementaryProfiler
```

Core should deal only with generic concepts:

```text
Session
Process
Frame
JVM
System telemetry
Clock correlation
Incident
Report
Collector quality
```

Current and future Argon optimizations enter through declarative Diagnostic Profiles when special emphasis is useful.

### 3.3 Fail independently

If one collector or one diagnostic profile cannot run:

```text
JFR unavailable
→ frame/system capture continues

GPU telemetry unavailable
→ frame/JVM capture continues

unknown Argon feature
→ generic capture continues

profile surface mismatch
→ that profile is skipped
```

A single integration failure must not invalidate the entire Probe session unless the requested mode fundamentally depends on it.

### 3.4 Low interference is a measured property

Probe must never claim to be “zero overhead”.

It distinguishes:

- **resource footprint** — Probe CPU, memory, wakeups, I/O;
- **measurement overhead** — actual change in the game's performance caused by Probe.

Only controlled Probe OFF / Probe ON experiments can establish the second item.

### 3.5 Raw evidence survives derived metrics

The report architecture is:

```text
Raw evidence
     ↓
Derived metrics
     ↓
Interpretation
```

Derived summaries may improve in future versions. Relevant raw evidence must remain available so an older report can be re-analyzed by a newer analyzer.

### 3.6 Prefer Locality

The implementation begins small.

Do not start with:

```text
ProviderFramework
CollectorManager
PluginManager
AdapterFactory
AnalysisService
ProfileRuntime
20 interfaces
```

Real boundaries should be introduced only after multiple concrete implementations require them.

---

## 4. Long-term architecture

```text
                         Minecraft JVM
                              │
             ┌────────────────┼────────────────┐
             │                │                │
             ↓                ↓                ↓
          PresentMon          JFR        Windows telemetry
             │                │                │
             └────────────────┼────────────────┘
                              ↓
                       Argon Probe Core
                              │
                    Clock correlation
                              │
              ┌───────────────┴───────────────┐
              ↓                               ↓
       Online session summary           Bounded raw rings
              │                               │
              └───────────────┬───────────────┘
                              ↓
                       Incident recorder
                              │
                              ↓
                         Evidence store
                              │
                       Minecraft exits
                              │
              ┌───────────────┼───────────────┐
              ↓               ↓               ↓
        environment       config hashes    log extract
              │               │               │
              └───────────────┼───────────────┘
                              ↓
                      argon-report-*.zip
```

Two configuration concepts sit outside the core:

```text
Diagnostic Profile
“What evidence matters for this optimization?”

Diagnostic Plan
“What exactly should this particular capture collect?”
```

A future external Deep Agent may be added only if real investigations show that generic external evidence cannot answer a necessary question.

---

## 5. Operating modes

Probe has three conceptually separate modes.

### 5.1 Bench

Purpose:

> Determine whether candidate code measurably changes performance.

Bench must minimize observer effect.

Default collection:

- PresentMon minimal frame metrics;
- very low frequency process/system telemetry;
- environment fingerprint;
- collector quality metadata.

Default disabled:

- JFR;
- incident escalation;
- deep method instrumentation;
- allocation tracing;
- automatic diagnostic snapshots.

Bench answers:

> Did performance change?

It does **not** answer:

> Why did performance change?

Target engineering budget:

- aim for no repeatable median-frame-time regression above approximately 0.5%;
- P99 must not show repeatable collector-induced degradation.

These are design targets, not assumed facts. They must be verified using Probe's own overhead benchmark procedure.

### 5.2 Scout

Purpose:

> Automatically gather broad, low-cost evidence while the developer uses Minecraft normally.

Default collection:

- PresentMon minimal frame data;
- bounded JFR continuous recording using a low-overhead configuration;
- low-frequency process/system telemetry;
- online frame statistics;
- incident detection;
- bounded incident windows;
- runtime environment metadata.

Scout does **not** dynamically attach a heavy profiler when a stutter occurs.

An incident freezes relevant evidence around the event; it does not automatically turn the current session into Deep mode.

Target engineering budget:

- aim for approximately ≤1% repeatable median-frame-time impact;
- no repeatable meaningful P99 degradation.

Again, this requires benchmark verification before release.

### 5.3 Deep

Purpose:

> Answer one targeted question that Scout could not answer.

Deep is a separate experiment.

It is driven by a Diagnostic Plan and may enable:

- shorter JFR sample period;
- allocation sampling;
- lock/contention detail;
- selected Windows tracing;
- a future temporary external Java agent where unavoidable.

Deep may have materially higher overhead and is never used to establish final performance gains.

An initial acceptable engineering envelope can be 2–5% overhead for short targeted captures, but the actual value must be measured and reported.

---

## 6. Collector strategy

### 6.1 PresentMon collector

PresentMon provides the external frame/present evidence.

Initial Scout/Bench should request only metrics that materially help diagnosis:

- frame/present timestamp;
- CPU frame duration where available;
- displayed/presented frame timing;
- GPU duration where available;
- present mode / dropped-present evidence where useful.

Do not poll dozens of hardware metrics merely because they exist.

Telemetry polling and ETW flush frequencies should be deliberately conservative. Probe is not an overlay and does not need to update a graph every few milliseconds.

Initial direction:

```text
hardware/system telemetry: ~1 Hz where practical
ETW/session flush: coarse enough to avoid unnecessary wakeups
```

The exact values are benchmark parameters, not permanent constants.

PresentMon capability discovery must be used where possible. Unsupported metrics should be omitted rather than treated as fatal.

Collector invariants:

- do not write one CSV row to disk for every normal frame;
- do not force synchronous disk flushes during gameplay;
- report lost/missing events;
- preserve enough timing metadata to correlate incidents later.

### 6.2 JVM/JFR collector

Scout uses JFR as the primary JVM evidence source.

The intended model is:

```text
continuous low-cost recording
+
bounded history
```

rather than:

```text
continuous heavy profiler
```

Initial Scout direction:

- low-overhead/default-style JFR configuration;
- bounded `maxAge`;
- bounded `maxSize`;
- execution samples;
- GC events;
- heap summaries where low cost;
- thread/monitor information where justified.

Initial target history:

```text
approximately 3–5 minutes
64–128 MiB budget
```

These are starting points. Actual buffer size, event loss, and overhead must be measured.

Bench defaults to JFR OFF.

If JFR cannot attach:

```text
JFR = unavailable
capture_quality records this
PresentMon/system capture continues
```

Do not terminate the whole session.

### 6.3 System collector

Initial Windows system/process telemetry should remain deliberately low frequency.

Candidate 1 Hz signals:

- Minecraft process CPU;
- Probe process CPU;
- Minecraft working set/private memory;
- Probe working set/private memory;
- thread count;
- relevant process I/O deltas;
- GPU utilization where reliable;
- VRAM use where reliable;
- CPU/GPU clocks or temperatures only if the chosen backend can provide them without material cost.

Hardware telemetry must be treated as optional capability.

Do not make vendor-specific GPU APIs a requirement for generic capture.

---

## 7. Frame data retention

A long Minecraft session can contain millions of frames. Storing every frame permanently by default is unnecessary.

Probe should use two data paths.

### 7.1 Bounded raw ring

Keep recent per-frame records in memory:

```text
initial target: 120 seconds
```

Old ordinary frames are overwritten.

### 7.2 Online whole-session statistics

Maintain compact aggregate statistics for the complete session:

- frame count;
- mean/median;
- percentile sketch or equivalent bounded estimator;
- P95 / P99 / P99.9;
- long-frame counts;
- histogram;
- incident count;
- min/max with appropriate caution.

No per-frame disk formatting is needed for normal gameplay.

### 7.3 Incident preservation

When an incident is detected, preserve a bounded raw frame window, for example:

```text
T - 5 s
through
T + 10 s
```

The exact window is configurable.

The goal is to retain the evidence that explains abnormal behavior, not every ordinary frame from a multi-hour session.

---

## 8. Incident detection

An incident is a bounded period of abnormal performance, not “one slow frame = one incident”.

### 8.1 Dynamic baseline

Do not use only a fixed threshold such as:

```text
frametime > 33.3 ms
```

Probe should maintain a recent stable baseline using robust statistics such as:

- rolling median;
- median absolute deviation (MAD);
- recent high percentiles;
- an absolute minimum threshold.

Conceptually:

```text
trigger_threshold =
max(
    absolute_floor,
    rolling_median + K × MAD
)
```

The exact algorithm and constants require validation on real Minecraft workloads.

### 8.2 Warm-up state

Minecraft startup, world entry, shader compile, JIT, and chunk loading are not automatically representative steady-state gameplay.

Session state should distinguish at least:

```text
STARTING
WARMUP
STEADY
```

Automatic incident detection should not treat expected startup/loading behavior as ordinary steady-state stutter evidence.

This state does not need game injection. Initial transitions can be conservative and may rely on elapsed time plus observed frame stability; later work may refine them using external evidence.

### 8.3 Hysteresis and merging

Consecutive abnormal frames should be merged into one incident.

An incident records:

```text
start
peak
recovery
duration
severity inputs
trigger reason
```

### 8.4 Cooldown and capture budget

Incident preservation is rate-limited. A sustained bad scene must not cause repeated expensive evidence finalization.

A session may detect many anomalies but only retain full raw evidence for a limited number of incidents.

The selection policy can preserve:

- worst incidents;
- first representative incident;
- selected later incidents with materially different characteristics.

All anomalies can still contribute to aggregate statistics.

### 8.5 No immediate heavy escalation

When an incident occurs:

```text
record marker
preserve ring data
continue Scout
```

Do not immediately attach a new heavy profiler.

Any expensive file finalization should preferably happen after recovery or at session end, not at the exact stutter peak.

---

## 9. Clock correlation

Different collectors do not necessarily share one native clock.

For Windows, Probe uses QPC as the primary Probe/Present timeline, but other sources are mapped into that timeline through explicit clock correlation.

A correlation record conceptually contains:

```json
{
  "qpc": 0,
  "utc": "...",
  "source_clock": "...",
  "source_timestamp": 0,
  "estimated_uncertainty_us": 0
}
```

Requirements:

- record QPC frequency;
- establish startup anchors;
- preserve source-native timestamps;
- periodically refresh correlation during long sessions if needed;
- record estimated synchronization uncertainty;
- never pretend microsecond ordering is known when the correlation precision does not support it.

This allows reliable questions such as:

> Did a 12 ms GC pause overlap the 80 ms frame incident?

without pretending to know unsupported sub-microsecond event ordering.

---

## 10. Diagnostic Profiles

A Diagnostic Profile is declarative project knowledge.

It answers:

> For this optimization area, which already available evidence should receive special attention?

Profiles are **not executable scripts**.

Preferred format:

```text
JSON
```

No `.py`, `.ps1`, arbitrary shell, or arbitrary code execution in profiles.

Example:

```json
{
  "profile_schema": 1,
  "id": "argon.iris.expression-cache",
  "requires": {
    "classes": [
      "io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformEvaluationPlanner"
    ]
  },
  "evidence": [
    "frames",
    "jfr.execution",
    "jfr.gc",
    "jfr.allocation"
  ]
}
```

Profile compatibility must come from real capability/surface checks where possible.

Avoid:

```text
version.startsWith(...)
```

Prefer checks such as:

- class/package present;
- collector capability present;
- known metadata surface present.

If a profile does not match:

```text
profile = skipped
generic capture = continues
```

Minecraft-version-specific knowledge remains isolated:

```text
profiles/
└── minecraft/
    ├── 26.2/
    └── 26.3/
```

Do not turn the 26.2 profile into a large future-version conditional tree.

Argon optimization work should use stable project-level IDs where useful, for example:

```text
iris.uniform-upload-dedup
iris.expression-cache
chunk.rebuild-cache
entity.transform-cache
```

These IDs can connect Issue, PR, Diagnostic Profile, benchmark evidence, and reports. A Java class rename should not destroy historical continuity.

---

## 11. Diagnostic Plans

A Diagnostic Plan is a per-experiment capture request.

It answers:

> What should this specific second run collect?

A plan can be generated after reviewing a Scout report.

Example:

```json
{
  "plan_schema": 1,
  "profile": "argon.iris.expression-cache",
  "duration_seconds": 30,
  "jfr": {
    "execution_sample_period_ms": 20,
    "allocation_sampling": true
  }
}
```

Profile and Plan are intentionally different:

```text
Profile = long-lived project knowledge
Plan    = one targeted experiment
```

A Plan must not silently change Bench mode into a heavy profile. Deep captures are explicit.

---

## 12. Future external Deep Agent

Generic JFR/PresentMon/system evidence cannot expose every possible internal semantic metric.

For example, a future optimization may care about:

- cache hit count;
- avoided rebuild count;
- deduplicated driver calls;
- batch size;
- internal revision miss rate.

An external observer cannot infer these values reliably if they exist only inside Argon code.

Therefore the architecture reserves, but does not yet implement, an optional external Java Deep Agent.

Rules:

- not part of Argon Mod;
- never shipped in the normal Argon JAR;
- never used by Bench;
- never attached automatically by Scout;
- used only by explicit Deep plans;
- instruments the smallest requested surface;
- removed when the Deep session ends;
- its overhead is measured separately.

Do **not** implement this agent until a real investigation proves generic collection insufficient.

---

## 13. Forward compatibility with future Argon work

This is a hard requirement.

A future Argon version must not require a Probe Core rewrite simply because new optimizations were added.

### 13.1 Generic-first rule

For every new optimization:

1. ask whether generic PresentMon/JFR/system evidence already makes it observable;
2. if yes, Probe changes are unnecessary;
3. if special emphasis is useful, add/update a declarative Profile;
4. if generic external evidence still cannot answer the required question, use a targeted Deep Plan;
5. only then consider the external Deep Agent.

### 13.2 Unknown Argon versions

An unknown Argon version must behave like:

```text
Generic frame capture      ACTIVE
Generic JFR Scout          ACTIVE if capability exists
System telemetry           ACTIVE
Known matching profiles    ACTIVE
Unknown/nonmatching        SKIPPED
```

Never:

```text
Unknown Argon version
→ Probe refuses to run
```

### 13.3 Future Minecraft versions

Probe Core should remain broadly version-agnostic.

Version-sensitive knowledge goes into explicit Minecraft profile directories instead of accumulating `if (mcVersion ...)` branches throughout the core.

---

## 14. Capability negotiation

At session start, Probe builds a Capability Snapshot.

Example:

```text
PresentMon              YES
JFR attach              YES
JFR allocation sample   YES
GPU telemetry           YES
Deep agent              NOT INSTALLED

Minecraft               26.2
Argon                    detected

Profiles:
generic.minecraft        ACTIVE
argon.iris.uniforms      ACTIVE
argon.chunk.rebuild      NOT PRESENT
```

Capabilities are written into the report.

A missing optional capability reduces evidence coverage; it does not silently masquerade as complete capture.

---

## 15. Environment fingerprint

Benchmark and diagnostic evidence is only useful if the runtime environment is known.

Capture relevant values such as:

- Minecraft version;
- Fabric Loader;
- Fabric API;
- Argon version/commit if available;
- Sodium version;
- Iris version;
- installed mod inventory and versions;
- shader pack identity;
- relevant shader/settings hashes;
- resolution;
- render distance;
- simulation distance;
- VSync;
- FPS limit;
- Java version;
- JVM arguments;
- CPU;
- GPU;
- RAM;
- graphics driver;
- Windows build.

Heavy inventory work should preferably occur after Minecraft exits:

- JAR hashing;
- configuration hashing;
- report compression;
- full mod inventory normalization.

Do not waste benchmark-time CPU and I/O calculating hashes that can wait.

---

## 16. Bench comparison

Bench comparison must treat environment mismatch as a first-class result.

Before calculating optimization deltas, compare critical fingerprints.

Potential invalidating differences include:

- Minecraft;
- Java;
- Fabric Loader/API;
- Sodium;
- Iris;
- mod set;
- shader pack/profile;
- render/simulation distance;
- resolution;
- VSync/FPS cap;
- JVM arguments;
- GPU driver.

If an important difference exists:

```text
ENVIRONMENT DIFFERENCE DETECTED
```

Do not quietly present a clean “+7% performance” result.

Primary benchmark candidates:

- median frame time;
- P95;
- P99;
- P99.9;
- 1% low;
- 0.1% low;
- frame-count thresholds such as >16.67 / >33.3 / >50 / >100 ms;
- CPU frame time;
- GPU frame time.

Supporting explanation metrics may include:

- GC pauses;
- allocation rate;
- process CPU;
- memory;
- telemetry quality.

Average FPS is secondary.

---

## 17. Probe overhead validation

Probe itself requires a benchmark gate.

### 17.1 Paired sequence

Prefer multiple paired/alternating runs rather than a single OFF/ON pair.

Example:

```text
OFF
ON
ON
OFF
```

repeated across multiple matched scenarios.

This helps reduce drift from temperature, CPU/GPU boost, JIT, background activity, and laptop power behavior.

### 17.2 Bench target

Initial engineering target:

```text
no repeatable >~0.5% median-frame-time regression
no repeatable meaningful P99 regression
```

### 17.3 Scout target

Initial engineering target:

```text
approximately <=1% repeatable median-frame-time impact
no repeatable meaningful P99 regression
```

These are not single-run pass/fail cliffs. Use repeated paired runs and uncertainty.

### 17.4 Deep

Deep overhead is reported, not hidden.

Deep data must never be used as final optimization benchmark evidence.

### 17.5 Resource footprint

Every report should also record Probe's own:

- CPU time;
- peak memory;
- I/O bytes;
- selected wakeup/collection indicators where practical.

Resource footprint is useful quality evidence but is not a substitute for an OFF/ON overhead benchmark.

---

## 18. Capture quality

Every report includes explicit capture quality.

Candidate fields:

```json
{
  "capture_quality": "GOOD",
  "present_events_lost": 0,
  "jfr_available": true,
  "jfr_dump_failures": 0,
  "time_sync_uncertainty_us": 0,
  "probe_cpu_time_ms": 0,
  "probe_memory_peak_bytes": 0,
  "probe_io_bytes": 0,
  "incident_windows_saved": 0
}
```

Possible quality states:

```text
GOOD
DEGRADED
INVALID
```

Examples:

- lost PresentMon events → DEGRADED;
- clock correlation unavailable → DEGRADED;
- requested Bench frame source unavailable → INVALID;
- JFR unavailable during Scout → DEGRADED but frame evidence may remain valid.

Do not silently discard collection failures.

---

## 19. Crash finalization

If the Minecraft JVM exits abnormally, Probe should finalize a crash-oriented report when possible.

Potential evidence:

- final frame ring;
- JFR bounded history;
- relevant `latest.log` excerpts;
- `crash-reports/` entry;
- `hs_err_pid*.log` if present;
- environment fingerprint;
- process exit status.

Report:

```text
session_status = CRASHED
```

Crash capture is evidence collection only; Probe does not claim to diagnose every JVM/GPU/driver failure automatically.

---

## 20. Privacy and data minimization

Probe reports are intended to be shared for analysis, so privacy is a first-class constraint.

Default redaction/omission:

- Windows username;
- absolute user-home path;
- account/session/authentication tokens;
- server addresses where practical;
- chat content;
- complete private logs;
- unrelated application data.

Normalize paths such as:

```text
C:\Users\Example\...
→ %USERPROFILE%\...
```

Do not upload reports automatically.

Full raw `latest.log` should require explicit opt-in. The default report should extract only relevant technical sections and errors.

---

## 21. Report format

First versions use ordinary formats:

```text
ZIP
JSON
CSV
JFR
```

Do not invent a custom binary container until there is a real requirement.

Example:

```text
argon-report-20260928-143100.zip
│
├── manifest.json
├── summary.json
├── quality.json
├── environment.json
├── capabilities.json
├── incidents.json
│
├── frames/
│   ├── incident-001.csv
│   └── incident-002.csv
│
├── telemetry/
│   └── system.csv
│
├── jfr/
│   └── continuous.jfr
│
└── logs/
    └── minecraft-relevant.log
```

Not every report contains every directory.

The manifest should identify:

- report schema;
- Probe version;
- mode;
- start/end time;
- session status;
- collectors used;
- collector/backend versions;
- active profiles;
- relevant collection parameters.

Collector provenance should contain enough information to reproduce or understand capture behavior.

---

## 22. Schema compatibility

Do not build the report around today's Phase A/Phase B fields.

Bad:

```json
{
  "phase_a": {},
  "phase_b": {}
}
```

Preferred top-level shape:

```json
{
  "report_schema": 1,
  "probe_version": "...",
  "mode": "SCOUT",
  "session": {},
  "environment": {},
  "capabilities": {},
  "quality": {},
  "collectors": [],
  "profiles": [],
  "incidents": []
}
```

Rules:

- add fields compatibly when possible;
- unknown fields are ignored by older readers;
- unknown profiles do not invalidate the report;
- breaking changes increment the relevant schema;
- Collector, Profile, Plan, and Report schemas version independently.

Raw evidence should remain readable even when derived analysis logic changes.

---

## 23. Probe versioning

Probe is a developer tool and should not be forced to share the Argon Mod release number.

Reports record both identities independently:

```text
Argon version
Argon commit
Probe version
Report schema
```

This allows Probe to evolve without implying a new Mod release, and allows one Probe build to inspect multiple compatible Argon builds.

A concrete Probe release-numbering policy can be chosen when the first executable is published; it does not need to be invented during the design-only stage.

---

## 24. Implementation direction

Initial platform:

```text
Windows first
```

Recommended implementation direction:

```text
Rust native executable
```

Reasons:

- small standalone runtime footprint;
- no additional JVM competing with Minecraft;
- appropriate Windows API access;
- predictable memory use;
- suitable for long-running low-frequency collection.

Do not force Rust if a specific integration proves substantially safer with a minimal native shim. Avoid introducing Python/Electron as the production runtime simply for development convenience, because their runtime footprint works against the tool's purpose.

External dependencies must be pinned and license-reviewed before vendoring or redistribution.

---

## 25. Development phases

### Phase P0 — Frame evidence prototype

Goal:

> Prove that a standalone collector can capture useful frame evidence without materially perturbing Minecraft.

Implement only:

- Minecraft PID detection;
- PresentMon minimal collection;
- QPC-based Probe timeline;
- 120-second frame ring;
- online frame statistics;
- basic environment snapshot;
- capability snapshot;
- quality metadata;
- report ZIP generation.

Do not implement JFR, Profiles, Plans, GUI, or Deep.

P0 exit gate:

- compile verified;
- package verified;
- launches on target Windows environment;
- identifies Minecraft reliably in tested cases;
- captures frame data;
- report reopens/parses correctly;
- Probe OFF/ON overhead benchmark completed;
- no unsupported claim of <=0.5% overhead without measurements.

### Phase P1 — Scout JVM evidence

Add:

- low-overhead continuous JFR;
- bounded JFR history;
- system/process telemetry;
- incident detector;
- incident raw windows;
- clock correlation;
- crash finalization;
- privacy redaction.

P1 exit gate:

- runtime verified;
- incident capture verified;
- clock-correlation quality measured;
- JFR unavailability fallback verified;
- Scout overhead benchmark completed;
- no meaningful repeatable P99 regression attributable to Probe in the validation workload.

### Phase P2 — Bench comparison and profiles

Add:

- Bench environment comparison;
- declarative Diagnostic Profile schema;
- Minecraft-version profile isolation;
- profile surface/capability validation;
- report/profile schema tests.

P2 exit gate:

- known matching profile activates;
- missing profile surface skips cleanly;
- unknown Argon version still performs generic capture;
- MC version profiles remain isolated;
- A/B comparison refuses or flags materially mismatched environments.

### Phase P3 — Diagnostic plans

Add:

- per-experiment Plan schema;
- targeted JFR configuration;
- bounded Deep sessions;
- explicit Deep quality/overhead reporting.

Do not add a Java agent unless a real investigation requires it.

### Phase P4 — External Deep Agent, only if justified

Potentially add:

- temporary attach;
- narrowly selected method/event instrumentation;
- semantic counters that cannot be externally inferred.

This phase is optional and may never be required.

---

## 26. Validation vocabulary

Probe follows the same strict language as Argon.

Possible states include:

- compile verified;
- package verified;
- structure/schema verified;
- runtime verified;
- capture-quality verified;
- overhead benchmark verified;
- comparison methodology verified.

Examples of invalid claims:

```text
“it builds, therefore it has <1% overhead”
“JFR started, therefore capture is correct”
“a profile JSON parses, therefore the profile is runtime-compatible”
“one ON run was faster, therefore Probe has no overhead”
```

---

## 27. CI strategy

As implementation grows, CI should cover only what CI can actually establish.

Candidate checks:

- build executable;
- schema validation;
- report parser round-trip;
- fixture incident detection;
- privacy-redaction fixtures;
- profile surface fixture checks;
- unsupported capability fallback tests;
- unknown profile compatibility;
- packaged artifact verification.

CI must not claim:

- real Minecraft runtime compatibility;
- real PresentMon event correctness;
- GPU telemetry correctness;
- measured overhead;
- visual/game behavior.

Those require runtime validation.

---

## 28. Future Argon optimization workflow

For every future Argon optimization:

```text
Reproduce
    ↓
Probe Scout / existing evidence
    ↓
Hot path identified?
    ├─ yes → source/overlap analysis
    └─ no  → targeted Diagnostic Plan
                    ↓
                Deep evidence
                    ↓
             source/overlap analysis
                    ↓
            minimal safe implementation
                    ↓
                Probe Bench
                    ↓
             behavior/visual checks
                    ↓
             compatibility testing
                    ↓
                   merge
```

Observability question for each new optimization:

> Can existing generic evidence measure the problem and its result?

If yes, no Probe feature work is required.

If no:

1. first consider a declarative Profile;
2. then a Diagnostic Plan;
3. only as a last resort consider a Deep Agent.

---

## 29. Initial folder evolution

At the design stage, keep everything local:

```text
tools/argon-probe/
└── README.md
```

When implementation begins and real boundaries appear, the likely shape is:

```text
tools/argon-probe/
├── README.md
├── src/
├── profiles/
├── schemas/
└── tests/
```

Do not create these directories merely to make the project look complete before code exists.

---

## 30. Decisions fixed by this design baseline

The following are deliberate baseline decisions:

1. Argon Probe is independent from the Fabric mod.
2. Probe Core contains no hard-coded current Argon optimization logic.
3. Windows is the first implementation/validation platform.
4. Bench and Scout are distinct modes.
5. Bench defaults to JFR OFF.
6. Scout uses bounded low-cost JVM evidence, not continuous heavy profiling.
7. Incidents preserve evidence but do not automatically trigger Deep profiling.
8. Normal frames live primarily in a bounded in-memory ring and aggregate statistics.
9. Profiles are declarative and non-executable.
10. Unknown Argon features do not disable generic capture.
11. Minecraft-version-specific profile knowledge is isolated by version.
12. A future Deep Agent remains external and is deferred until justified.
13. Report schema is generic and forward-compatible.
14. Raw evidence is retained where it materially enables future re-analysis.
15. Probe overhead is benchmarked, never assumed.
16. Reports are local by default and never uploaded automatically.
17. Privacy redaction is part of the report pipeline, not an afterthought.

---

## 31. Open questions intentionally deferred

These should be answered by implementation evidence, not design speculation:

- exact PresentMon API integration strategy;
- exact telemetry/flush periods;
- exact frame-ring duration;
- exact MAD/incident constants;
- exact JFR event list;
- exact JFR history size;
- exact Rust crate choices;
- whether a small native interoperability shim is needed;
- whether GPU vendor telemetry provides enough value to justify extra dependencies;
- whether a Deep Agent is ever necessary;
- Probe's final public release-numbering policy;
- cross-platform backends beyond Windows.

Each should be resolved at the phase where it becomes a real engineering boundary.

---

## 32. Success criterion

Argon Probe is successful when the normal development experience becomes:

```text
1. Start Probe.
2. Use Minecraft normally or run a defined benchmark.
3. Stop/exit.
4. Send one report.
5. Determine the next evidence/source investigation from that report.
```

without requiring the developer to manually coordinate several profiling tools, and without materially changing the performance behavior being investigated.

The tool must improve the quality and repeatability of Argon's evidence, not merely increase the amount of telemetry collected.
