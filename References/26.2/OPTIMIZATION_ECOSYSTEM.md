# Optimization Ecosystem Baseline

Argon targets **Minecraft 26.2 / Fabric** and is designed to coexist with the established optimization ecosystem.

## Core references

| Project | 26.2 reference | Primary area | Upstream |
| --- | --- | --- | --- |
| Sodium | `mc26.2-0.9.2` | renderer, terrain rendering, rendering buffers, client rendering performance | https://github.com/CaffeineMC/sodium |
| Iris | `1.11.4+mc26.2` source baseline | shader loading, shader render pipeline, Sodium integration | https://github.com/IrisShaders/Iris |
| Lithium | `mc26.2-0.25.2` | game logic, entities, physics, ticking, world logic, shapes | https://github.com/CaffeineMC/lithium |
| FerriteCore | `9.0.0-fabric` | retained memory and memory-oriented data/layout optimizations | https://github.com/malte0811/FerriteCore |
| ImmediatelyFast | `1.16.4+26.2-fabric` | immediate-mode rendering, GUI and related client rendering hot paths | https://github.com/RaphiMC/ImmediatelyFast |
| EntityCulling | `1.10.5` (26.2 Fabric build) | entity / block-entity visibility culling | https://github.com/tr7zw/EntityCulling |

These projects are compatibility and overlap references, not default Argon dependencies.

A proposed Argon patch should answer which vanilla code it touches, whether an upstream mod already solves the bottleneck, whether the transforms overlap, and whether Argon should disable the patch when that mod is present.

## Initial ownership boundaries

### Sodium

Treat Sodium as the primary owner of deep renderer replacement work. Be cautious around chunk/terrain renderer replacement, terrain buffers, render graph/culling internals, and backend-specific OpenGL/Vulkan paths.

### Iris

Treat Iris as the primary shader-loader/render-pipeline compatibility target.

Before Argon changes rendering state, framebuffer lifetime, render passes, shader-visible data, vertex formats, or Sodium-facing render hooks, inspect Iris as well as Sodium.

Argon rendering optimizations should be tested in at least these two states:

- Sodium without shaders;
- Sodium + Iris with the project's shader compatibility baseline enabled.

See `SHADER_COMPATIBILITY.md`.

### Lithium

Inspect Lithium before changing entity ticking/lookup, collision and VoxelShape hot paths, block/scheduled ticks, explosions, mob AI/sensors, or world-logic caches.

### FerriteCore

Inspect FerriteCore before retained-memory deduplication or structural memory optimizations. Argon should preferentially target measured allocation churn, temporary objects, object lifetime, and GC pressure where it does not duplicate FerriteCore.

### ImmediatelyFast

Inspect ImmediatelyFast before changing immediate rendering, GUI batching, or similar client rendering paths.

### EntityCulling

Do not duplicate occlusion-based entity/block-entity culling. Non-occlusion work such as distance-based animation policies can remain separate if measurable and compatible.

## License awareness

These projects do not all use the same license. Before incorporating implementation detail beyond general ideas and API-compatible behavior, inspect the exact upstream license of the pinned revision.

Argon's default rule is to re-implement from first principles based on profiling, vanilla behavior, documentation, and independently derived design rather than copying third-party code.

## Updating versions

These versions are the initial 26.2 study baseline, not permanent dependency pins. When compatibility testing moves to a newer 26.2 patch release, update this table in the same change.
