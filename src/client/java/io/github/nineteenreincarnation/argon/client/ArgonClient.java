package io.github.nineteenreincarnation.argon.client;

import io.github.nineteenreincarnation.argon.Argon;
import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformDeduplicator;
import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import net.fabricmc.api.ClientModInitializer;

public final class ArgonClient implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        String minecraft = CompatibilityBaseline26_2.installedVersion("minecraft").orElse("missing");
        String iris = CompatibilityBaseline26_2.installedVersion("iris").orElse("missing");
        String sodium = CompatibilityBaseline26_2.installedVersion("sodium").orElse("missing");

        Argon.LOGGER.info(
            "Argon client baseline: Minecraft {}, Iris {} ({}), Sodium {}.",
            minecraft,
            iris,
            CompatibilityBaseline26_2.irisCompatibility(),
            sodium
        );

        Argon.LOGGER.info(
            "Iris uniform Phase 0 instrumentation: requested={}, active={}.",
            IrisUniformInstrumentation.isRequested(),
            IrisUniformInstrumentation.isEnabled()
        );

        Argon.LOGGER.info(
            "Iris uniform Phase A deduplication: requested={}, active={}.",
            IrisUniformDeduplicator.isRequested(),
            IrisUniformDeduplicator.isEnabled()
        );
    }
}
