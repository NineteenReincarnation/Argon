# Shader Compatibility Baseline — Minecraft 26.2

Argon's primary shader regression stack is:

- **Minecraft:** 26.2
- **Renderer:** Sodium 0.9.2
- **Shader loader:** Iris 1.11.4
- **Shader pack:** Spooklementary 2.0.4
- **Spooklementary commit:** `21a40924f608abc4d3035dd7daf401663ffb4d1f`

The exact Spooklementary 2.0.4 source is available at:

`third-party/spooklementary/`

## Minimum rendering regression matrix

Any Argon patch touching rendering state, framebuffers, passes, vertex formats, shader-visible data, Sodium hooks, or related client rendering behavior should be tested in at least:

1. Fabric/vanilla rendering where the patch applies;
2. Sodium with shaders disabled;
3. Sodium + Iris with shaders disabled;
4. Sodium + Iris + Spooklementary 2.0.4 enabled.

## Why Spooklementary is included

A shader pack is a real workload, not only a visual accessory. Rendering optimizations that appear correct without shaders can still break framebuffer sequencing, shadow passes, translucent rendering, shader assumptions, or Sodium/Iris integration.

Spooklementary 2.0.4 is therefore part of the Argon 26.2 regression baseline.

## License note

Spooklementary is distributed under the Complementary License Agreement. The source remains an upstream Git submodule rather than copied Argon-owned source. Its license and attribution remain inside that repository.
