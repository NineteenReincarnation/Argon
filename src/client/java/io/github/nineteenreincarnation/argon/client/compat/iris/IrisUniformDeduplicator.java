package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import it.unimi.dsi.fastutil.objects.Object2IntMap;

import java.util.IdentityHashMap;

public final class IrisUniformDeduplicator {
    private static final boolean REQUESTED =
        Boolean.parseBoolean(System.getProperty("argon.experimental.irisUniformDedup", "false"));

    private static final boolean ACTIVE =
        REQUESTED
            && CompatibilityBaseline26_2.isMinecraftTarget()
            && CompatibilityBaseline26_2.supportsIrisPhaseA();

    private static final IdentityHashMap<Object, IdentityHashMap<Object, Long>> PROGRAM_REVISIONS =
        new IdentityHashMap<>();

    private IrisUniformDeduplicator() {
    }

    public static boolean isRequested() {
        return REQUESTED;
    }

    public static boolean isEnabled() {
        return ACTIVE;
    }

    public static void onPipelineReset() {
        PROGRAM_REVISIONS.clear();
    }

    public static boolean tryPush(Object pass, Object mappedUniforms) {
        if (!ACTIVE) {
            return false;
        }

        if (!(mappedUniforms instanceof Object2IntMap<?> uniforms)) {
            return false;
        }

        for (Object2IntMap.Entry<?> entry : uniforms.object2IntEntrySet()) {
            if (!(entry.getKey() instanceof UniformState)) {
                return false;
            }
        }

        IdentityHashMap<Object, Long> uploaded =
            PROGRAM_REVISIONS.computeIfAbsent(pass, ignored -> new IdentityHashMap<>());

        for (Object2IntMap.Entry<?> entry : uniforms.object2IntEntrySet()) {
            Object uniform = entry.getKey();
            UniformState state = (UniformState) uniform;
            long revision = state.argon$revision();
            Long uploadedRevision = uploaded.get(uniform);
            boolean required = uploadedRevision == null || uploadedRevision.longValue() != revision;

            IrisUniformInstrumentation.onUploadCheck(required);

            if (required) {
                state.argon$push(entry.getIntValue());
                uploaded.put(uniform, revision);
            }
        }

        return true;
    }

    public interface UniformState {
        long argon$revision();

        void argon$incrementRevision();

        void argon$push(int location);
    }
}
