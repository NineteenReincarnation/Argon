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
- Verify the packaged development JAR contains the expected 26.2 integration classes and resources.

### Validation

- CI compile validation: complete.
- CI package-content validation: complete.
- Iris 26.2 structure validation: complete for 1.11.0, 1.11.1, 1.11.2, and 1.11.4.
- In-game validation: not yet performed.
- Performance validation: not yet performed.
