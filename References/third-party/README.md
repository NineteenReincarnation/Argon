# Third-party source references

These directories are Git submodules pinned to source revisions used as Argon's **Minecraft 26.2 compatibility and overlap baseline**.

They are reference material. They are **not Argon dependencies**, and their code is not automatically licensed for incorporation into Argon.

| Directory | Upstream version | Pinned commit | License / note |
| --- | --- | --- | --- |
| `sodium/` | Sodium 0.9.2 for 26.2 | `6c26e7b7eded82ce5a1d27f9b147ce5d8de99b7a` | PolyForm Shield 1.0.0; treat as read-only architectural/compatibility reference |
| `lithium/` | Lithium 0.25.2 for 26.2 | `8940fbcb39bac29b7a3dc279fa61595cd9dc4deb` | LGPL-3.0 |
| `ferritecore/` | FerriteCore 9.0.0 | `0cef1f2add1f1329aa6e690e8e292acd625c5c6d` | MIT |
| `immediatelyfast/` | ImmediatelyFast 1.16.4 | `03d336242c8f965f9644f6700ee1e0c4c1657aa8` | LGPL-3.0 |
| `entityculling/` | EntityCulling 1.10.5 | `c87c146dcad35102f07a24a67e71ae1e4f23b507` | tr7zw Protective License; read and test compatibility, do not merge code into Argon |

## Getting the source

Clone Argon with:

```bash
git clone --recurse-submodules https://github.com/NineteenReincarnation/Argon.git
```

For an existing clone:

```bash
git submodule update --init --recursive
```

The pinned commit is part of the Argon repository tree. Running a normal submodule update will therefore restore the exact source revision listed above.

## How these sources are used

Before implementing a patch:

1. profile Minecraft 26.2;
2. identify the vanilla classes and methods involved;
3. search these source trees for transforms of the same code;
4. determine whether the upstream optimization already removes the bottleneck;
5. determine whether Argon's patch is additive, redundant, or conflicting;
6. record any compatibility gate needed by Argon.

Do not copy code simply because it is visible here. Check the upstream license and independently derive Argon's implementation.

## Updating a reference

A submodule pointer should only move intentionally. When it does, update `References/OPTIMIZATION_ECOSYSTEM.md` in the same change and explain why the new revision is a better 26.2 compatibility baseline.
