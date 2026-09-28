package io.github.nineteenreincarnation.argon.version.mc26_2;

import io.github.nineteenreincarnation.argon.Argon;
import org.objectweb.asm.tree.ClassNode;
import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;

import java.util.List;
import java.util.Set;

public final class Argon26_2MixinPlugin implements IMixinConfigPlugin {
    private static final String IRIS_MIXIN_SEGMENT = ".mc26_2.iris.";
    private static boolean loggedIrisDecision;

    @Override
    public void onLoad(String mixinPackage) {
    }

    @Override
    public String getRefMapperConfig() {
        return null;
    }

    @Override
    public boolean shouldApplyMixin(String targetClassName, String mixinClassName) {
        if (!mixinClassName.contains(IRIS_MIXIN_SEGMENT)) {
            return true;
        }

        boolean minecraftSupported = CompatibilityBaseline26_2.isMinecraftTarget();
        CompatibilityBaseline26_2.IrisCompatibility irisCompatibility =
            CompatibilityBaseline26_2.irisCompatibility();
        boolean apply = minecraftSupported && CompatibilityBaseline26_2.supportsIrisPhase0();

        if (!loggedIrisDecision) {
            loggedIrisDecision = true;

            String minecraft = CompatibilityBaseline26_2.installedVersion("minecraft").orElse("missing");
            String iris = CompatibilityBaseline26_2.installedVersion("iris").orElse("missing");

            if (apply) {
                Argon.LOGGER.info(
                    "Enabling Minecraft 26.2 Iris Phase 0 integration (Minecraft {}, Iris {}, compatibility {}).",
                    minecraft,
                    iris,
                    irisCompatibility
                );
            } else if (irisCompatibility == CompatibilityBaseline26_2.IrisCompatibility.NOT_INSTALLED) {
                Argon.LOGGER.info(
                    "Iris is not installed; Iris-specific Argon Mixins are disabled."
                );
            } else {
                Argon.LOGGER.warn(
                    "Disabling Iris-specific Argon Mixins: Minecraft {}, Iris {}, compatibility {}. Structure-verified Iris versions: {}.",
                    minecraft,
                    iris,
                    irisCompatibility,
                    CompatibilityBaseline26_2.IRIS_PHASE0_STRUCTURE_VERIFIED
                );
            }
        }

        return apply;
    }

    @Override
    public void acceptTargets(Set<String> myTargets, Set<String> otherTargets) {
    }

    @Override
    public List<String> getMixins() {
        return null;
    }

    @Override
    public void preApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {
    }

    @Override
    public void postApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {
    }
}
