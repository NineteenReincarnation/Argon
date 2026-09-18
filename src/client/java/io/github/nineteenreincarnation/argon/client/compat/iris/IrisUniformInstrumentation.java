package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;
import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.time.Instant;
import java.util.IdentityHashMap;
import java.util.Locale;
import java.util.Map;

public final class IrisUniformInstrumentation {
    private static final boolean REQUESTED =
        Boolean.parseBoolean(System.getProperty("argon.instrumentation.irisUniforms", "true"));

    private static final boolean ACTIVE =
        REQUESTED
            && CompatibilityBaseline26_2.isMinecraftTarget()
            && CompatibilityBaseline26_2.supportsIrisPhase0();

    private static final long REPORT_INTERVAL_NANOS =
        Math.max(1L, Long.getLong("argon.instrumentation.reportIntervalSeconds", 10L)) * 1_000_000_000L;

    private static final long WARMUP_NANOS =
        Math.max(0L, Long.getLong("argon.instrumentation.warmupSeconds", 5L)) * 1_000_000_000L;

    private static final Path CSV_PATH =
        FabricLoader.getInstance().getGameDir().resolve("argon").resolve("phase0-iris-uniforms.csv");

    private static final IdentityHashMap<Object, Boolean> SEEN_UNIFORMS = new IdentityHashMap<>();
    private static final IdentityHashMap<Object, SimulatedProgramState> PROGRAM_REVISIONS =
        new IdentityHashMap<>();

    private static boolean measurementStarted;
    private static boolean csvEnabled = true;
    private static long warmupUntilNanos;
    private static long intervalStartedNanos;
    private static long pipelineGeneration;

    private static long completedFrames;
    private static long evaluations;
    private static long changedEvaluations;
    private static long passPushes;
    private static long uploadChecks;
    private static long actualUploads;
    private static long simulatedUploadChecks;
    private static long simulatedRequiredUploads;
    private static long simulatedAvoidableUploads;
    private static long phaseAFastPathSkips;
    private static long phaseARevisionScans;
    private static long updateNanos;
    private static long pushNanos;

    private IrisUniformInstrumentation() {
    }

    public static boolean isRequested() {
        return REQUESTED;
    }

    public static boolean isEnabled() {
        return ACTIVE;
    }

    public static boolean isMeasuring() {
        return ACTIVE && measurementStarted;
    }

    public static void onPipelineReset() {
        if (!ACTIVE) {
            return;
        }

        pipelineGeneration++;
        SEEN_UNIFORMS.clear();
        PROGRAM_REVISIONS.clear();
        resetMeasurementCounters();

        measurementStarted = false;
        long now = System.nanoTime();
        warmupUntilNanos = now + WARMUP_NANOS;

        Argon.LOGGER.info(
            "[Phase 0][Iris uniforms] Pipeline state reset; generation={}, warm-up={} second(s).",
            pipelineGeneration,
            WARMUP_NANOS / 1_000_000_000L
        );
    }

    public static void onFrameStart() {
        if (!ACTIVE) {
            return;
        }

        long now = System.nanoTime();

        if (warmupUntilNanos == 0L) {
            warmupUntilNanos = now + WARMUP_NANOS;
        }

        if (!measurementStarted) {
            if (now < warmupUntilNanos) {
                return;
            }

            measurementStarted = true;
            intervalStartedNanos = now;
            Argon.LOGGER.info(
                "[Phase 0][Iris uniforms] Warm-up complete; generation={}, reporting every {} second(s), CSV={}.",
                pipelineGeneration,
                REPORT_INTERVAL_NANOS / 1_000_000_000L,
                CSV_PATH
            );
            return;
        }

        completedFrames++;

        if (now - intervalStartedNanos >= REPORT_INTERVAL_NANOS) {
            reportAndReset(now);
        }
    }

    public static void onEvaluation() {
        if (measurementStarted) {
            evaluations++;
        }
    }

    public static void onEvaluationResult(Object uniform, boolean changed) {
        if (!ACTIVE) {
            return;
        }

        SEEN_UNIFORMS.put(uniform, Boolean.TRUE);

        if (changed && measurementStarted) {
            changedEvaluations++;
        }
    }

    public static void onPassPush(Object pass, Object mappedUniforms) {
        if (!ACTIVE) {
            return;
        }

        if (measurementStarted) {
            passPushes++;
        }

        if (!(mappedUniforms instanceof Map<?, ?> uniforms)) {
            return;
        }

        SimulatedProgramState programState = PROGRAM_REVISIONS.get(pass);
        if (programState == null || programState.locationMapIdentity() != mappedUniforms) {
            programState = new SimulatedProgramState(mappedUniforms);
            PROGRAM_REVISIONS.put(pass, programState);
        }

        IdentityHashMap<Object, Long> uploaded = programState.uploadedRevisions();

        for (Object uniform : uniforms.keySet()) {
            boolean revisionAvailable = uniform instanceof IrisUniformDeduplicator.UniformState;
            long revision = revisionAvailable
                ? ((IrisUniformDeduplicator.UniformState) uniform).argon$revision()
                : 0L;
            Long uploadedRevision = uploaded.get(uniform);
            boolean required =
                !revisionAvailable || uploadedRevision == null || uploadedRevision.longValue() != revision;

            if (measurementStarted) {
                simulatedUploadChecks++;
                if (required) {
                    simulatedRequiredUploads++;
                } else {
                    simulatedAvoidableUploads++;
                }
            }

            if (required && revisionAvailable) {
                uploaded.put(uniform, revision);
            }
        }
    }

    public static void onUploadCheck(boolean uploaded) {
        if (!measurementStarted) {
            return;
        }

        uploadChecks++;
        if (uploaded) {
            actualUploads++;
        }
    }

    public static void onPhaseAFastPath() {
        if (measurementStarted) {
            phaseAFastPathSkips++;
        }
    }

    public static void onPhaseARevisionScan() {
        if (measurementStarted) {
            phaseARevisionScans++;
        }
    }

    public static void onUpdateDuration(long nanos) {
        if (measurementStarted) {
            updateNanos += nanos;
        }
    }

    public static void onPushDuration(long nanos) {
        if (measurementStarted) {
            pushNanos += nanos;
        }
    }

    private static void reportAndReset(long now) {
        long frames = Math.max(1L, completedFrames);

        Report report = new Report(
            Instant.now().toString(),
            pipelineGeneration,
            completedFrames,
            SEEN_UNIFORMS.size(),
            PROGRAM_REVISIONS.size(),
            perFrame(evaluations, frames),
            percent(changedEvaluations, evaluations),
            100.0D - percent(changedEvaluations, evaluations),
            perFrame(passPushes, frames),
            perFrame(uploadChecks, frames),
            perFrame(actualUploads, frames),
            perFrame(simulatedUploadChecks, frames),
            perFrame(simulatedRequiredUploads, frames),
            perFrame(simulatedAvoidableUploads, frames),
            percent(simulatedAvoidableUploads, simulatedUploadChecks),
            perFrame(phaseAFastPathSkips, frames),
            perFrame(phaseARevisionScans, frames),
            nanosPerFrameAsMicros(updateNanos, frames),
            nanosPerFrameAsMicros(pushNanos, frames)
        );

        Argon.LOGGER.info(
            "[Phase 0][Iris uniforms] generation={} frames={} uniforms={} programs={} eval/frame={} changed={}%, stable={}%, passPush/frame={}, actualUploads/frame={}, simulatedRequired/frame={}, simulatedAvoidable/frame={}, simulatedSkip={}%, phaseAFastSkip/frame={}, phaseAScan/frame={}, instrumentedUpdateUs/frame={}, instrumentedPushUs/frame={}",
            report.pipelineGeneration(),
            report.frames(),
            report.uniforms(),
            report.programs(),
            report.evaluationsPerFrame(),
            report.changedPercent(),
            report.stablePercent(),
            report.passPushesPerFrame(),
            report.actualUploadsPerFrame(),
            report.simulatedRequiredPerFrame(),
            report.simulatedAvoidablePerFrame(),
            report.simulatedSkipPercent(),
            report.phaseAFastPathSkipsPerFrame(),
            report.phaseARevisionScansPerFrame(),
            report.instrumentedUpdateUsPerFrame(),
            report.instrumentedPushUsPerFrame()
        );

        if (uploadChecks != simulatedUploadChecks) {
            Argon.LOGGER.debug(
                "[Phase 0][Iris uniforms] Actual upload checks ({}) differed from simulated checks ({}).",
                uploadChecks,
                simulatedUploadChecks
            );
        }

        if (IrisUniformDeduplicator.isEnabled() && actualUploads != simulatedRequiredUploads) {
            Argon.LOGGER.warn(
                "[Phase A][Iris uniforms] Revision consistency mismatch: actual required uploads={} simulated required uploads={}.",
                actualUploads,
                simulatedRequiredUploads
            );
        }

        appendCsv(report);

        resetMeasurementCounters();
        intervalStartedNanos = now;
    }

    private static void appendCsv(Report report) {
        if (!csvEnabled) {
            return;
        }

        try {
            Files.createDirectories(CSV_PATH.getParent());

            boolean writeHeader = Files.notExists(CSV_PATH) || Files.size(CSV_PATH) == 0L;
            StringBuilder output = new StringBuilder();

            if (writeHeader) {
                output.append(
                    "timestamp_utc,pipeline_generation,phase_a_enabled,argon_version,minecraft_version,iris_version,sodium_version," +
                    "frames,uniforms,programs,evaluations_per_frame,changed_percent,stable_percent," +
                    "pass_pushes_per_frame,actual_upload_checks_per_frame,actual_uploads_per_frame," +
                    "simulated_upload_checks_per_frame,simulated_required_per_frame,simulated_avoidable_per_frame," +
                    "simulated_skip_percent,phase_a_fast_path_skips_per_frame,phase_a_revision_scans_per_frame," +
                    "instrumented_update_us_per_frame,instrumented_push_us_per_frame\n"
                );
            }

            output.append(csv(report.timestampUtc())).append(',')
                .append(report.pipelineGeneration()).append(',')
                .append(IrisUniformDeduplicator.isEnabled()).append(',')
                .append(csv(CompatibilityBaseline26_2.installedVersion("argon").orElse("unknown"))).append(',')
                .append(csv(CompatibilityBaseline26_2.installedVersion("minecraft").orElse("unknown"))).append(',')
                .append(csv(CompatibilityBaseline26_2.installedVersion("iris").orElse("unknown"))).append(',')
                .append(csv(CompatibilityBaseline26_2.installedVersion("sodium").orElse("unknown"))).append(',')
                .append(report.frames()).append(',')
                .append(report.uniforms()).append(',')
                .append(report.programs()).append(',')
                .append(decimal(report.evaluationsPerFrame())).append(',')
                .append(decimal(report.changedPercent())).append(',')
                .append(decimal(report.stablePercent())).append(',')
                .append(decimal(report.passPushesPerFrame())).append(',')
                .append(decimal(report.actualUploadChecksPerFrame())).append(',')
                .append(decimal(report.actualUploadsPerFrame())).append(',')
                .append(decimal(report.simulatedUploadChecksPerFrame())).append(',')
                .append(decimal(report.simulatedRequiredPerFrame())).append(',')
                .append(decimal(report.simulatedAvoidablePerFrame())).append(',')
                .append(decimal(report.simulatedSkipPercent())).append(',')
                .append(decimal(report.phaseAFastPathSkipsPerFrame())).append(',')
                .append(decimal(report.phaseARevisionScansPerFrame())).append(',')
                .append(decimal(report.instrumentedUpdateUsPerFrame())).append(',')
                .append(decimal(report.instrumentedPushUsPerFrame()))
                .append('\n');

            Files.writeString(
                CSV_PATH,
                output,
                StandardCharsets.UTF_8,
                StandardOpenOption.CREATE,
                StandardOpenOption.APPEND
            );
        } catch (IOException e) {
            csvEnabled = false;
            Argon.LOGGER.warn(
                "[Phase 0][Iris uniforms] Failed to write CSV report to {}; CSV output disabled for this session.",
                CSV_PATH,
                e
            );
        }
    }

    private static String decimal(double value) {
        return String.format(Locale.ROOT, "%.6f", value);
    }

    private static String csv(String value) {
        return "\"" + value.replace("\"", "\"\"") + "\"";
    }

    public static void onPhaseAFallback() {
        if (!ACTIVE || !measurementStarted) {
            return;
        }

        resetMeasurementCounters();
        intervalStartedNanos = System.nanoTime();

        Argon.LOGGER.warn(
            "[Phase 0][Iris uniforms] Discarded the current measurement window because Phase A fell back to Iris' original upload path."
        );
    }

    private static void resetMeasurementCounters() {
        completedFrames = 0L;
        evaluations = 0L;
        changedEvaluations = 0L;
        passPushes = 0L;
        uploadChecks = 0L;
        actualUploads = 0L;
        simulatedUploadChecks = 0L;
        simulatedRequiredUploads = 0L;
        simulatedAvoidableUploads = 0L;
        phaseAFastPathSkips = 0L;
        phaseARevisionScans = 0L;
        updateNanos = 0L;
        pushNanos = 0L;
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

    private record SimulatedProgramState(
        Object locationMapIdentity,
        IdentityHashMap<Object, Long> uploadedRevisions
    ) {
        private SimulatedProgramState(Object locationMapIdentity) {
            this(locationMapIdentity, new IdentityHashMap<>());
        }
    }

    private record Report(
        String timestampUtc,
        long pipelineGeneration,
        long frames,
        int uniforms,
        int programs,
        double evaluationsPerFrame,
        double changedPercent,
        double stablePercent,
        double passPushesPerFrame,
        double actualUploadChecksPerFrame,
        double actualUploadsPerFrame,
        double simulatedUploadChecksPerFrame,
        double simulatedRequiredPerFrame,
        double simulatedAvoidablePerFrame,
        double simulatedSkipPercent,
        double phaseAFastPathSkipsPerFrame,
        double phaseARevisionScansPerFrame,
        double instrumentedUpdateUsPerFrame,
        double instrumentedPushUsPerFrame
    ) {
    }
}
