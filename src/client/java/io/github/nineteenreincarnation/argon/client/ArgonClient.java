package io.github.nineteenreincarnation.argon.client;

import io.github.nineteenreincarnation.argon.Argon;
import net.fabricmc.api.ClientModInitializer;

public final class ArgonClient implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        Argon.LOGGER.info("Argon client initialized.");
    }
}
