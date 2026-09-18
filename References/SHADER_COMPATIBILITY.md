# Shader Compatibility Baseline

Argon's primary shader compatibility target for Minecraft 26.2 is:

- **Loader:** Iris 1.11.4 for Minecraft 26.2
- **Renderer:** Sodium 0.9.2 for Minecraft 26.2
- **Shader pack:** Spooklementary
- **26.2 reference shader release:** Spooklementary v2.0.4
- **Shader upstream:** https://github.com/SpacEagle17/Spooklementary
- **Project page:** https://modrinth.com/shader/spooklementary

Spooklementary is an edit of Complementary Shaders and is distributed under the Complementary License. Its source is therefore referenced upstream rather than vendored into Argon.

## Why this baseline exists

Rendering optimizations can look correct with no shader pack enabled while breaking:

- framebuffer sequencing;
- shadow passes;
- translucent rendering;
- shader-visible uniforms or buffers;
- render-state assumptions;
- Sodium/Iris integration paths.

For any Argon patch that touches rendering, the minimum client regression matrix should include:

1. vanilla/Fabric rendering without Sodium where the patch applies;
2. Sodium with shaders disabled;
3. Sodium + Iris with shaders disabled;
4. Sodium + Iris + Spooklementary enabled.

If a rendering patch intentionally does not support one of these states, the limitation must be explicit and the patch should be compatibility-gated rather than failing silently.

## Version note

The shader pack used by an individual test machine may differ from the reference release. Benchmark notes must record the exact local Spooklementary version whenever shaders are enabled.
