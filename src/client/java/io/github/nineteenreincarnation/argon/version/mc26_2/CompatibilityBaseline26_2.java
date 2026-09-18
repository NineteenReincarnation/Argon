package io.github.nineteenreincarnation.argon.version.mc26_2;

import net.fabricmc.loader.api.FabricLoader;

import java.util.Optional;

public final class CompatibilityBaseline26_2 {
    public static final String MINECRAFT = "26.2";
    public static final String IRIS = "1.11.4+mc26.2";
    public static final String SODIUM = "0.9.2+mc26.2";
    public static final String SPOOKLEMENTARY = "2.0.4";

    private CompatibilityBaseline26_2() {
    }

    public static boolean isMinecraftTarget() {
        return hasExactVersion("minecraft", MINECRAFT);
    }

    public static boolean isSupportedIris() {
        return hasExactVersion("iris", IRIS);
    }

    public static Optional<String> installedVersion(String modId) {
        return FabricLoader.getInstance()
            .getModContainer(modId)
            .map(container -> container.getMetadata().getVersion().getFriendlyString());
    }

    private static boolean hasExactVersion(String modId, String expected) {
        return installedVersion(modId).map(expected::equals).orElse(false);
    }
}
