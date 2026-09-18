package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;
import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Reference2LongOpenHashMap;
import it.unimi.dsi.fastutil.objects.Reference2ObjectOpenHashMap;

public final class IrisUniformDeduplicator {
    private static final long MISSING_REVISION = Long.MIN_VALUE;

    private static final boolean REQUESTED =
        Boolean.parseBoolean(System.getProperty("argon.experimental.irisUniformDedup", "false"));

    private static final boolean COMPATIBLE =
        CompatibilityBaseline26_2.isMinecraftTarget()
            && CompatibilityBaseline26_2.supportsIrisPhaseA();

    private static final Reference2ObjectOpenHashMap<Object, Reference2LongOpenHashMap<Object>> PROGRAM_REVISIONS =
        new Reference2ObjectOpenHashMap<>();

    private static boolean operational = true;

    private IrisUniformDeduplicator() {
    }

    public static boolean isRequested() {
        return REQUESTED;
    }

    public static boolean isEnabled() {
        return REQUESTED && COMPATIBLE && operational;
    }

    public static void onPipelineReset() {
        PROGRAM_REVISIONS.clear();
    }

    public static boolean tryPush(Object pass, Object mappedUniforms) {
        if (!isEnabled()) {
            return false;
        }

        if (mappedUniforms == null) {
            return false;
        }

        if (!(mappedUniforms instanceof Object2IntMap<?> uniforms)) {
            disableForSession("Iris custom-uniform location map no longer implements Object2IntMap.");
            return false;
        }

        Reference2LongOpenHashMap<Object> uploaded = PROGRAM_REVISIONS.get(pass);
        if (uploaded == null) {
            uploaded = new Reference2LongOpenHashMap<>();
            uploaded.defaultReturnValue(MISSING_REVISION);
            PROGRAM_REVISIONS.put(pass, uploaded);
        }

        try {
            for (Object2IntMap.Entry<?> entry : uniforms.object2IntEntrySet()) {
                Object uniform = entry.getKey();
                UniformState state = (UniformState) uniform;
                long revision = state.argon$revision();
                long uploadedRevision = uploaded.getLong(uniform);
                boolean required = uploadedRevision == MISSING_REVISION || uploadedRevision != revision;

                IrisUniformInstrumentation.onUploadCheck(required);

                if (required) {
                    state.argon$push(entry.getIntValue());
                    uploaded.put(uniform, revision);
                }
            }
        } catch (ClassCastException e) {
            disableForSession("An Iris cached uniform did not expose Argon's revision state.");
            return false;
        }

        return true;
    }

    private static void disableForSession(String reason) {
        if (!operational) {
            return;
        }

        operational = false;
        PROGRAM_REVISIONS.clear();

        Argon.LOGGER.error(
            "Disabling experimental Iris uniform deduplication for this session: {} Falling back to Iris' original upload path.",
            reason
        );
    }

    public interface UniformState {
        long argon$revision();

        void argon$incrementRevision();

        void argon$push(int location);
    }
}
