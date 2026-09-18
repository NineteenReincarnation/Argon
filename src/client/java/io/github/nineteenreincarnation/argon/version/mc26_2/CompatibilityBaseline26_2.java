package io.github.nineteenreincarnation.argon.version.mc26_2;

import net.fabricmc.loader.api.FabricLoader;

import java.util.Optional;
import java.util.Set;

public final class CompatibilityBaseline26_2 {
    public static final String MINECRAFT = "26.2";

    public static final String IRIS_PRIMARY = "1.11.4+mc26.2";
    public static final Set<String> IRIS_PHASE0_STRUCTURE_VERIFIED = Set.of(
        "1.11.0+mc26.2",
        "1.11.1+mc26.2",
        "1.11.2+mc26.2",
        IRIS_PRIMARY
    );

    public static final Set<String> IRIS_PHASE_A_STRUCTURE_VERIFIED =
        Set.copyOf(IRIS_PHASE0_STRUCTURE_VERIFIED);

    public static final Set<String> IRIS_PHASE_B_STRUCTURE_VERIFIED =
        Set.of(IRIS_PRIMARY);

    public static final String SODIUM_PRIMARY = "0.9.2+mc26.2";
    public static final String SPOOKLEMENTARY_PRIMARY = "2.0.4";

    private CompatibilityBaseline26_2() {
    }

    public static boolean isMinecraftTarget() {
        return hasExactVersion("minecraft", MINECRAFT);
    }

    public static IrisCompatibility irisCompatibility() {
        Optional<String> installed = installedVersion("iris");

        if (installed.isEmpty()) {
            return IrisCompatibility.NOT_INSTALLED;
        }

        String version = installed.get();

        if (IRIS_PRIMARY.equals(version)) {
            return IrisCompatibility.PRIMARY_BASELINE;
        }

        if (IRIS_PHASE0_STRUCTURE_VERIFIED.contains(version)) {
            return IrisCompatibility.STRUCTURE_VERIFIED;
        }

        return IrisCompatibility.UNSUPPORTED;
    }

    public static boolean supportsIrisPhase0() {
        return switch (irisCompatibility()) {
            case PRIMARY_BASELINE, STRUCTURE_VERIFIED -> true;
            case NOT_INSTALLED, UNSUPPORTED -> false;
        };
    }

    public static boolean supportsIrisPhaseA() {
        return installedVersion("iris")
            .map(IRIS_PHASE_A_STRUCTURE_VERIFIED::contains)
            .orElse(false);
    }

    public static boolean supportsIrisPhaseB() {
        return installedVersion("iris")
            .map(IRIS_PHASE_B_STRUCTURE_VERIFIED::contains)
            .orElse(false);
    }

    public static Optional<String> installedVersion(String modId) {
        return FabricLoader.getInstance()
            .getModContainer(modId)
            .map(container -> container.getMetadata().getVersion().getFriendlyString());
    }

    private static boolean hasExactVersion(String modId, String expected) {
        return installedVersion(modId).map(expected::equals).orElse(false);
    }

    public enum IrisCompatibility {
        NOT_INSTALLED,
        STRUCTURE_VERIFIED,
        PRIMARY_BASELINE,
        UNSUPPORTED
    }
}
