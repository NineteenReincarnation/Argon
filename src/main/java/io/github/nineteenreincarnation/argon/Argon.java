package io.github.nineteenreincarnation.argon;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class Argon implements ModInitializer {
    public static final String MOD_ID = "argon";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        LOGGER.info("Argon initialized.");
    }
}
