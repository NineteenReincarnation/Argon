# First Runtime Test — Argon v0.1.0 / Minecraft 26.2

This is the first in-game validation procedure for the Iris custom-uniform Phase 0 build.

## Current validation boundary

Before this test, Argon has:

- compile validation;
- packaged-JAR validation;
- Iris 1.11.0 / 1.11.1 / 1.11.2 / 1.11.4 target-surface validation.

It does **not** yet have in-game, visual, or performance validation.

## Recommended Windows path

From the Argon repository root:

```powershell
.\scripts\run-phase0.ps1
```

The script:

1. initializes Git submodules if the pinned Spooklementary source is missing;
2. starts the development client with the primary shader runtime stack;
3. uses Fabric API 0.160.0+26.2, Sodium 0.9.2, Iris 1.11.4, and Spooklementary 2.0.4;
4. leaves Argon's Phase 0 instrumentation enabled;
5. after Minecraft exits, extracts the relevant lines from `run/logs/latest.log`;
6. writes `run/argon-phase0-summary.txt`.

Generated `run/` data is ignored by Git.

## First checkpoint: startup only

Before collecting performance data, verify the client can:

- reach the title screen;
- enter a world;
- render with Spooklementary enabled;
- exit normally.

Expected Argon log lines include the installed baseline and:

```text
Iris uniform Phase 0 instrumentation: requested=true, active=true
```

and an Iris integration line indicating a structure-verified/primary baseline compatibility profile.

If the game crashes or the instrumentation is inactive, stop there. Do not treat that session as benchmark data.

## Second checkpoint: pipeline activity

After entering a world with shaders enabled, wait at least 15 seconds.

The log should begin reporting entries shaped like:

```text
[Phase 0][Iris uniforms] frames=... uniforms=... programs=... eval/frame=... changed=... stable=... actualUploads/frame=... simulatedRequired/frame=... simulatedAvoidable/frame=... simulatedSkip=...
```

The first five seconds after a pipeline reset are a warm-up window and are intentionally excluded from measurement counters.

## Initial measurement scenarios

Once startup/runtime behavior is confirmed:

1. **Static outdoor** — stand still in a representative Overworld scene with stable weather.
2. **Continuous movement** — move and rotate the camera through changing terrain/biomes.
3. **Weather transition** — clear to rain/thunder.
4. **Entity stress** — a repeatable entity-heavy scene with entity shadows enabled.

Do not change shader settings during a single measurement interval.

## What to preserve

For each capture, keep:

- exact Argon commit;
- Iris/Sodium versions;
- Spooklementary profile/settings;
- render distance and simulation distance;
- scenario;
- the extracted Phase 0 summary lines.

The first runtime test is primarily for correctness and instrumentation validity. FPS gains are **not** expected yet because Phase 0 does not implement the optimization.


## Structured result file

During the run, Argon also appends interval data to:

```text
run/argon/phase0-iris-uniforms.csv
```

Keep this CSV together with `run/argon-phase0-summary.txt`. The CSV is the preferred input for later comparison/analysis; the text summary is mainly for quick inspection.
