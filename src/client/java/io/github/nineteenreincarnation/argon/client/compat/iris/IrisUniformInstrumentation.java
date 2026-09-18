package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;

public final class IrisUniformInstrumentation {
    private static final boolean ENABLED =
        Boolean.parseBoolean(System.getProperty("argon.instrumentation.irisUniforms", "true"));

    private static final long REPORT_INTERVAL_NANOS =
        Math.max(1L, Long.getLong("argon.instrumentation.reportIntervalSeconds", 10L)) * 1_000_000_000L;

    private static boolean frameStarted;
    private static long intervalStartedNanos;

    private static long completedFrames;
    private static long evaluations;
    private static long changedEvaluations;
    private static long passPushes;
    private static long uploadChecks;
    private static long actualUploads;
    private static long updateNanos;
    private static long pushNanos;

    private IrisUniformInstrumentation() {
    }

    public static boolean isEnabled() {
        return ENABLED;
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

    public static void onEvaluationResult(boolean changed) {
        if (ENABLED && changed) {
            changedEvaluations++;
        }
    }

    public static void onPassPush() {
        if (ENABLED) {
            passPushes++;
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
        double changedPercent = percent(changedEvaluations, evaluations);
        double stablePercent = 100.0D - changedPercent;
        double uploadedPercent = percent(actualUploads, uploadChecks);

        Argon.LOGGER.info(
            "[Phase 0][Iris uniforms] frames={} eval/frame={} changed={}%, stable={}%, passPush/frame={}, uploadChecks/frame={}, actualUploads/frame={}, uploaded={}%, updateUs/frame={}, pushUs/frame={}",
            completedFrames,
            perFrame(evaluations, frames),
            changedPercent,
            stablePercent,
            perFrame(passPushes, frames),
            perFrame(uploadChecks, frames),
            perFrame(actualUploads, frames),
            uploadedPercent,
            nanosPerFrameAsMicros(updateNanos, frames),
            nanosPerFrameAsMicros(pushNanos, frames)
        );

        completedFrames = 0L;
        evaluations = 0L;
        changedEvaluations = 0L;
        passPushes = 0L;
        uploadChecks = 0L;
        actualUploads = 0L;
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
