# Third-party source references — Minecraft 26.2

These Git submodules are pinned to the source revisions used as Argon's **Minecraft 26.2 compatibility, overlap, and regression baseline**.

They are reference material, not automatic Argon dependencies. Each upstream project's license still applies.

| Directory | Reference | Pinned commit | Purpose / license note |
| --- | --- | --- | --- |
| `sodium/` | Sodium 0.9.2 | `6c26e7b7eded82ce5a1d27f9b147ce5d8de99b7a` | Renderer baseline; PolyForm Shield 1.0.0 |
| `iris/` | Iris 1.11.4 for 26.2 | `f61d950f556f3962d0e2e29c270bc1865572b35d` | Shader-loader/render-pipeline baseline; LGPL-3.0 |
| `lithium/` | Lithium 0.25.2 | `8940fbcb39bac29b7a3dc279fa61595cd9dc4deb` | Game-logic baseline; LGPL-3.0 |
| `ferritecore/` | FerriteCore 9.0.0 | `0cef1f2add1f1329aa6e690e8e292acd625c5c6d` | Memory baseline; MIT |
| `immediatelyfast/` | ImmediatelyFast 1.16.4 | `03d336242c8f965f9644f6700ee1e0c4c1657aa8` | Immediate/client rendering baseline; LGPL-3.0 |
| `entityculling/` | EntityCulling 1.10.5 | `c87c146dcad35102f07a24a67e71ae1e4f23b507` | Culling baseline; tr7zw Protective License |
| `spooklementary/` | Spooklementary 2.0.4 | `21a40924f608abc4d3035dd7daf401663ffb4d1f` | Primary real-world shader workload; Complementary License Agreement 1.6 |

## Getting all reference source

Clone Argon with:

```bash
git clone --recurse-submodules https://github.com/NineteenReincarnation/Argon.git
```

For an existing clone:

```bash
git pull
git submodule sync --recursive
git submodule update --init --recursive
```

## Usage rule

Before implementing a patch, inspect the relevant upstream source to determine whether the hotspot is already optimized and whether Argon would be additive, redundant, or conflicting.

Do not copy implementation code merely because the source is present. Check the exact license and independently derive Argon's implementation.
