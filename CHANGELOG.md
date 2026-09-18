# Changelog

Argon follows semantic versioning for the mod release line and records the Minecraft target in build metadata.

## 0.1.0 — Unreleased

Target: Minecraft 26.2 / Fabric

### Development

- Bootstrap the Minecraft 26.2 Fabric project.
- Establish CI builds and version/project-management rules.
- Isolate Minecraft 26.2 integration and exact-version Iris Mixins.
- Add Phase 0 instrumentation for the Iris custom-uniform pipeline.
- Add a non-invasive per-program revision simulation to estimate safe uniform-upload deduplication.
- Add warm-up handling for stable Phase 0 measurement windows.
- Use the pinned Iris 1.11.4 source revision for Mixin/source audits.
- Use Spooklementary 2.0.4 as the primary planned shader workload.
- Verify the packaged development JAR contains the expected 26.2 integration classes and resources.

### Validation

- CI compile validation: complete.
- CI package-content validation: complete.
- In-game validation: not yet performed.
- Performance validation: not yet performed.
