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
        boolean irisSupported = CompatibilityBaseline26_2.isSupportedIris();
        boolean apply = minecraftSupported && irisSupported;

        if (!loggedIrisDecision) {
            loggedIrisDecision = true;
            String minecraft = CompatibilityBaseline26_2.installedVersion("minecraft").orElse("missing");
            String iris = CompatibilityBaseline26_2.installedVersion("iris").orElse("missing");

            if (apply) {
                Argon.LOGGER.info(
                    "Enabling Minecraft 26.2 Iris integration (Minecraft {}, Iris {}).",
                    minecraft,
                    iris
                );
            } else {
                Argon.LOGGER.warn(
                    "Disabling Minecraft 26.2 Iris integration: expected Minecraft {} / Iris {}, found Minecraft {} / Iris {}.",
                    CompatibilityBaseline26_2.MINECRAFT,
                    CompatibilityBaseline26_2.IRIS,
                    minecraft,
                    iris
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
