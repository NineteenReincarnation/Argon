# Changelog

Argon follows semantic versioning for the mod release line and records the Minecraft target in build metadata.

## 0.1.0 — Unreleased

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
- Verify the packaged development JAR contains the expected 26.2 integration classes and resources.\n- Add a reproducible development runtime profile for Sodium 0.9.2 + Iris 1.11.4 + Spooklementary 2.0.4.\n- Add machine-readable CSV output for Phase 0 measurement intervals.\n- Implement experimental, default-off per-program Iris custom-uniform upload deduplication for the primary 1.11.4 baseline.\n- Use identity/primitive revision maps on the Phase A hot path and fall back to Iris automatically if runtime structural assumptions fail.\n- Combine program-remap invalidation with a global change-epoch fast path that can skip the entire uniform map when no custom uniform changed.\n- Add a program-level update-sequence fast path that bypasses repeated pushes in the same update cycle.\n- Add a changed-set incremental path so continuously used programs inspect only uniforms that actually changed in the current update when that is cheaper than a full scan.

### Validation

- CI compile validation: complete.
- CI package-content validation: complete.
- Iris 26.2 structure validation: complete for 1.11.0, 1.11.1, 1.11.2, and 1.11.4.
- In-game validation: not yet performed.
- Performance validation: not yet performed.
