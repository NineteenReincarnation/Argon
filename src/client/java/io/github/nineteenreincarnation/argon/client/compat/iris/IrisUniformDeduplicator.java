package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;
import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import it.unimi.dsi.fastutil.objects.Reference2LongOpenHashMap;
import it.unimi.dsi.fastutil.objects.Reference2ObjectOpenHashMap;

public final class IrisUniformDeduplicator {
    private static final long MISSING_REVISION = Long.MIN_VALUE;
    private static final long UNSYNCED_UPDATE = Long.MIN_VALUE;

    private static final boolean REQUESTED =
        Boolean.parseBoolean(System.getProperty("argon.experimental.irisUniformDedup", "false"));

    private static final boolean COMPATIBLE =
        CompatibilityBaseline26_2.isMinecraftTarget()
            && CompatibilityBaseline26_2.supportsIrisPhaseA();

    private static final Reference2ObjectOpenHashMap<Object, ProgramState> PROGRAMS =
        new Reference2ObjectOpenHashMap<>();

    private static final ObjectArrayList<Object> CHANGED_THIS_UPDATE =
        new ObjectArrayList<>();

    private static boolean operational = true;
    private static boolean updateInProgress;
    private static long updateSequence;

    private IrisUniformDeduplicator() {
    }

    public static boolean isRequested() {
        return REQUESTED;
    }

    public static boolean isEnabled() {
        return REQUESTED && COMPATIBLE && operational;
    }

    public static void onUniformUpdateStart() {
        if (!isEnabled()) {
            return;
        }

        advanceUpdateSequence();
        CHANGED_THIS_UPDATE.clear();
        updateInProgress = true;
    }

    public static void onUniformUpdateEnd() {
        if (isEnabled()) {
            updateInProgress = false;
        }
    }

    public static void onUniformChanged(Object uniform) {
        if (!isEnabled()) {
            return;
        }

        if (!updateInProgress) {
            // The audited Iris path updates custom uniforms from CustomUniforms.update().
            // If a future Iris version mutates one outside that boundary, invalidate all
            // program state rather than making a partial changed-set assumption.
            PROGRAMS.clear();
            CHANGED_THIS_UPDATE.clear();
            return;
        }

        CHANGED_THIS_UPDATE.add(uniform);
    }

    public static void onPipelineReset() {
        PROGRAMS.clear();
        CHANGED_THIS_UPDATE.clear();
        updateSequence = 0L;
        updateInProgress = false;
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

        ProgramState program = PROGRAMS.get(pass);

        if (program == null || program.locationMapIdentity != mappedUniforms) {
            program = new ProgramState(mappedUniforms);
            PROGRAMS.put(pass, program);
            fullScan(program, uniforms);
        } else if (program.syncedUpdate == updateSequence) {
            IrisUniformInstrumentation.onPhaseAFastPath();
            return true;
        } else if (program.syncedUpdate == updateSequence - 1L) {
            if (CHANGED_THIS_UPDATE.isEmpty()) {
                IrisUniformInstrumentation.onPhaseAFastPath();
            } else if (CHANGED_THIS_UPDATE.size() < uniforms.size()) {
                incrementalScan(program, uniforms);
            } else {
                fullScan(program, uniforms);
            }
        } else {
            fullScan(program, uniforms);
        }

        if (!isEnabled()) {
            return false;
        }

        program.syncedUpdate = updateSequence;
        return true;
    }

    private static void incrementalScan(ProgramState program, Object2IntMap<?> uniforms) {
        IrisUniformInstrumentation.onPhaseAIncrementalScan();

        try {
            for (Object uniform : CHANGED_THIS_UPDATE) {
                if (!uniforms.containsKey(uniform)) {
                    continue;
                }

                UniformState state = (UniformState) uniform;
                long revision = state.argon$revision();

                // The program was synchronized in the immediately previous update.
                // Every uniform in CHANGED_THIS_UPDATE therefore requires an upload
                // when this program maps that uniform.
                IrisUniformInstrumentation.onUploadCheck(true);
                state.argon$push(uniforms.getInt(uniform));
                program.revisions.put(uniform, revision);
            }
        } catch (ClassCastException e) {
            disableForSession("An Iris cached uniform did not expose Argon's revision state.");
        }
    }

    private static void fullScan(ProgramState program, Object2IntMap<?> uniforms) {
        IrisUniformInstrumentation.onPhaseAFullScan();

        try {
            for (Object2IntMap.Entry<?> entry : uniforms.object2IntEntrySet()) {
                Object uniform = entry.getKey();
                UniformState state = (UniformState) uniform;
                long revision = state.argon$revision();
                long uploadedRevision = program.revisions.getLong(uniform);
                boolean required =
                    uploadedRevision == MISSING_REVISION || uploadedRevision != revision;

                IrisUniformInstrumentation.onUploadCheck(required);

                if (required) {
                    state.argon$push(entry.getIntValue());
                    program.revisions.put(uniform, revision);
                }
            }
        } catch (ClassCastException e) {
            disableForSession("An Iris cached uniform did not expose Argon's revision state.");
        }
    }

    private static void advanceUpdateSequence() {
        updateSequence++;

        if (updateSequence == UNSYNCED_UPDATE) {
            PROGRAMS.clear();
            updateSequence = 0L;
        }
    }

    private static void disableForSession(String reason) {
        if (!operational) {
            return;
        }

        operational = false;
        PROGRAMS.clear();
        CHANGED_THIS_UPDATE.clear();
        updateInProgress = false;
        IrisUniformInstrumentation.onPhaseAFallback();

        Argon.LOGGER.error(
            "Disabling experimental Iris uniform deduplication for this session: {} Falling back to Iris' original upload path.",
            reason
        );
    }

    private static final class ProgramState {
        private final Object locationMapIdentity;
        private final Reference2LongOpenHashMap<Object> revisions =
            new Reference2LongOpenHashMap<>();
        private long syncedUpdate = UNSYNCED_UPDATE;

        private ProgramState(Object locationMapIdentity) {
            this.locationMapIdentity = locationMapIdentity;
            revisions.defaultReturnValue(MISSING_REVISION);
        }
    }

    public interface UniformState {
        long argon$revision();

        void argon$incrementRevision();

        void argon$push(int location);
    }
}
