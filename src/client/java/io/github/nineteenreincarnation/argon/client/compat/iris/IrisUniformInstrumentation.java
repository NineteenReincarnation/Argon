package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;

import java.util.IdentityHashMap;
import java.util.Map;

public final class IrisUniformInstrumentation {
    private static final boolean ENABLED =
        Boolean.parseBoolean(System.getProperty("argon.instrumentation.irisUniforms", "true"));

    private static final long REPORT_INTERVAL_NANOS =
        Math.max(1L, Long.getLong("argon.instrumentation.reportIntervalSeconds", 10L)) * 1_000_000_000L;

    private static final IdentityHashMap<Object, Long> UNIFORM_REVISIONS = new IdentityHashMap<>();
    private static final IdentityHashMap<Object, IdentityHashMap<Object, Long>> PROGRAM_REVISIONS =
        new IdentityHashMap<>();

    private static boolean frameStarted;
    private static long intervalStartedNanos;

    private static long completedFrames;
    private static long pipelineResets;

    private static long evaluations;
    private static long changedEvaluations;

    private static long passPushes;
    private static long uploadChecks;
    private static long actualUploads;

    private static long simulatedUploadChecks;
    private static long simulatedRequiredUploads;
    private static long simulatedAvoidableUploads;

    private static long updateNanos;
    private static long pushNanos;

    private IrisUniformInstrumentation() {
    }

    public static boolean isEnabled() {
        return ENABLED;
    }

    public static void onPipelineReset() {
        if (!ENABLED) {
            return;
        }

        UNIFORM_REVISIONS.clear();
        PROGRAM_REVISIONS.clear();
        pipelineResets++;
    }

    public static void onFrameStart() {
        if (!ENABLED) {
            return;
        }

        long now = System.nanoTime();

        if (!frameStarted) {
            frameStarted = true;
            intervalStartedNanos = now;
            Argon.LOGGER.info(
                "[Phase 0][Iris uniforms] Instrumentation started; reporting every {} second(s).",
                REPORT_INTERVAL_NANOS / 1_000_000_000L
            );
            return;
        }

        completedFrames++;

        if (now - intervalStartedNanos >= REPORT_INTERVAL_NANOS) {
            reportAndReset(now);
        }
    }

    public static void onEvaluation() {
        if (ENABLED) {
            evaluations++;
        }
    }

    public static void onEvaluationResult(Object uniform, boolean changed) {
        if (!ENABLED) {
            return;
        }

        if (changed) {
            changedEvaluations++;
            UNIFORM_REVISIONS.put(uniform, UNIFORM_REVISIONS.getOrDefault(uniform, 0L) + 1L);
        } else {
            UNIFORM_REVISIONS.putIfAbsent(uniform, 0L);
        }
    }

    public static void onPassPush(Object pass, Object mappedUniforms) {
        if (!ENABLED) {
            return;
        }

        passPushes++;

        if (!(mappedUniforms instanceof Map<?, ?> uniforms)) {
            return;
        }

        IdentityHashMap<Object, Long> uploaded =
            PROGRAM_REVISIONS.computeIfAbsent(pass, ignored -> new IdentityHashMap<>());

        for (Object uniform : uniforms.keySet()) {
            simulatedUploadChecks++;

            long revision = UNIFORM_REVISIONS.getOrDefault(uniform, 0L);
            Long uploadedRevision = uploaded.get(uniform);

            if (uploadedRevision == null || uploadedRevision.longValue() != revision) {
                simulatedRequiredUploads++;
                uploaded.put(uniform, revision);
            } else {
                simulatedAvoidableUploads++;
            }
        }
    }

    public static void onUploadCheck(boolean uploaded) {
        if (!ENABLED) {
            return;
        }

        uploadChecks++;
        if (uploaded) {
            actualUploads++;
        }
    }

    public static void onUpdateDuration(long nanos) {
        if (ENABLED) {
            updateNanos += nanos;
        }
    }

    public static void onPushDuration(long nanos) {
        if (ENABLED) {
            pushNanos += nanos;
        }
    }

    private static void reportAndReset(long now) {
        long frames = Math.max(1L, completedFrames);

        Argon.LOGGER.info(
            "[Phase 0][Iris uniforms] frames={} resets={} eval/frame={} changed={}%, stable={}%, passPush/frame={}, actualUploads/frame={}, simulatedRequired/frame={}, simulatedAvoidable/frame={}, simulatedSkip={}%, updateUs/frame={}, pushUs/frame={}",
            completedFrames,
            pipelineResets,
            perFrame(evaluations, frames),
            percent(changedEvaluations, evaluations),
            100.0D - percent(changedEvaluations, evaluations),
            perFrame(passPushes, frames),
            perFrame(actualUploads, frames),
            perFrame(simulatedRequiredUploads, frames),
            perFrame(simulatedAvoidableUploads, frames),
            percent(simulatedAvoidableUploads, simulatedUploadChecks),
            nanosPerFrameAsMicros(updateNanos, frames),
            nanosPerFrameAsMicros(pushNanos, frames)
        );

        if (uploadChecks != simulatedUploadChecks) {
            Argon.LOGGER.debug(
                "[Phase 0][Iris uniforms] Actual upload checks ({}) differed from simulated checks ({}).",
                uploadChecks,
                simulatedUploadChecks
            );
        }

        completedFrames = 0L;
        pipelineResets = 0L;
        evaluations = 0L;
        changedEvaluations = 0L;
        passPushes = 0L;
        uploadChecks = 0L;
        actualUploads = 0L;
        simulatedUploadChecks = 0L;
        simulatedRequiredUploads = 0L;
        simulatedAvoidableUploads = 0L;
        updateNanos = 0L;
        pushNanos = 0L;
        intervalStartedNanos = now;
    }

    private static double perFrame(long value, long frames) {
        return (double) value / (double) frames;
    }

    private static double nanosPerFrameAsMicros(long nanos, long frames) {
        return ((double) nanos / (double) frames) / 1_000.0D;
    }

    private static double percent(long numerator, long denominator) {
        if (denominator == 0L) {
            return 0.0D;
        }
        return ((double) numerator * 100.0D) / (double) denominator;
    }
}
