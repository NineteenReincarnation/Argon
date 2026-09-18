# Spooklementary 2.0.4 — Custom Uniform Audit

Pinned source:

```text
SpacEagle17/Spooklementary
21a40924f608abc4d3035dd7daf401663ffb4d1f
shaders/shaders.properties
```

This document records architecture-relevant observations only. The shader-pack source remains the authoritative reference.

## Inventory

The pinned `shaders.properties` contains **30 custom uniform/variable definitions** in the custom-expression block.

Of those definitions, **18 directly use Iris/OptiFine-style `smooth(...)` stateful expressions**.

The stateful group covers several real workload categories:

- cave / eye-light adaptation;
- precipitation and rain state;
- Nether / biome blending;
- movement/startup smoothing;
- frame-time smoothing;
- eye-brightness smoothing;
- lightning response;
- water/altitude transition state.

Other definitions include frame-counter modulo values, camera-position deltas, movement predicates, blindness/darkness combination, and ground-state data.

## Phase B implication

A naive rule such as:

```text
input value unchanged -> skip expression evaluation
```

is not safe for this workload.

A `smooth(...)` expression has temporal/stateful behavior: its output may continue evolving even when the immediate source value is unchanged. Any dependency-aware evaluation optimization must therefore distinguish at least:

```text
pure expression
stateful/time-dependent expression
unknown/conservative expression
```

Stateful and unknown expressions remain scheduled unless a stronger correctness proof exists.

## Candidate future strategy

If Phase A measurements show that `CustomUniforms.update()` remains a material CPU cost:

1. classify expression/function nodes by purity/statefulness;
2. propagate the classification through the existing Iris dependency graph;
3. track revisions only for pure upstream inputs;
4. skip a pure derived expression only when all dependency revisions are unchanged;
5. continue evaluating stateful/time-dependent chains normally;
6. validate shader-visible values against the baseline before any default enablement.

This is a research direction only. Phase B remains blocked on real Phase A / Phase 0 profiling data.
