# Changelog

Argon uses the project-specific `X.Y.Z+<minecraft-version>` scheme: X is the major content version, Y is the release line, Z is the development iteration, and the suffix records the Minecraft target.

## Unreleased — Minecraft 26.2

Target: Minecraft 26.2 / Fabric

### Development

- Bootstrap the Minecraft 26.2 Fabric project.
- Establish CI builds and version/project-management rules.
- Isolate Minecraft 26.2 integration and version-sensitive Mixins.
- Add Phase 0 instrumentation for the Iris custom-uniform pipeline.
- Add a non-invasive per-program revision simulation to estimate safe uniform-upload deduplication.
- Add warm-up handling for stable Phase 0 measurement windows.
- Add a published-Iris structural compatibility matrix for Minecraft 26.2.
- Enable Phase 0 instrumentation for structure-verified Iris 1.11.0, 1.11.1, 1.11.2, and 1.11.4.
- Keep Iris 1.11.4 + Sodium 0.9.2 + Spooklementary 2.0.4 as the primary planned benchmark baseline.
- Verify the packaged development JAR contains the expected 26.2 integration classes and resources.
- Add a reproducible development runtime profile for Sodium 0.9.2 + Iris 1.11.4 + Spooklementary 2.0.4.
- Add machine-readable CSV output for Phase 0 measurement intervals.
- Implement experimental, default-off per-program Iris custom-uniform upload deduplication for the primary 1.11.4 baseline.
- Use identity/primitive revision maps on the Phase A hot path and fall back to Iris automatically if runtime structural assumptions fail.
- Combine program-remap invalidation with a global change-epoch fast path that can skip the entire uniform map when no custom uniform changed.
- Add a program-level update-sequence fast path that bypasses repeated pushes in the same update cycle.
- Add a changed-set incremental path so continuously used programs inspect only uniforms that actually changed in the current update when that is cheaper than a full scan.

- Merge program-remap safety with a three-tier custom-uniform push path: O(1) fast skip, changed-set incremental push, and full revision fallback.

- Add an experimental, default-off Phase B dependency-revision evaluation cache for conservatively classified pure Iris custom expressions.

- Update the Minecraft 26.2 development baseline to Fabric API 0.161.0+26.2.

- Add a simulation-only Phase B mode that measures cache opportunity without skipping Iris evaluation and detects dependency/classification mismatches.

- Extend Argon Probe P0 capture quality with PresentMon ETW loss diagnostics, including final stderr warning parsing without storing raw stderr.
- Avoid enabling PresentMon `--track_etw_status` in the stdout-CSV path because the audited upstream implementation emits periodic non-CSV status text to stdout.
- Stage Probe reports to a temporary ZIP, flush them, reopen and validate required JSON/CSV structure, then atomically publish the final report.\n\n### Validation

- CI compile validation: complete.
- CI package-content validation: complete.
- Iris 26.2 structure validation: complete for 1.11.0, 1.11.1, 1.11.2, and 1.11.4.
- In-game validation: not yet performed.
- Performance validation: not yet performed.
