package io.github.nineteenreincarnation.argon.client.compat.iris;

import io.github.nineteenreincarnation.argon.Argon;
import io.github.nineteenreincarnation.argon.version.mc26_2.CompatibilityBaseline26_2;
import it.unimi.dsi.fastutil.objects.Reference2ObjectOpenHashMap;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;

public final class IrisUniformEvaluationPlanner {
    private static final boolean REQUESTED =
        Boolean.parseBoolean(System.getProperty("argon.experimental.irisUniformEvaluation", "false"));

    private static final boolean COMPATIBLE =
        CompatibilityBaseline26_2.isMinecraftTarget()
            && CompatibilityBaseline26_2.supportsIrisPhaseB();

    private static final Reference2ObjectOpenHashMap<Object, CandidateState> CANDIDATES =
        new Reference2ObjectOpenHashMap<>();

    private static boolean operational = true;
    private static ReflectionModel reflection;

    private IrisUniformEvaluationPlanner() {
    }

    public static boolean isRequested() {
        return REQUESTED;
    }

    public static boolean isEnabled() {
        return REQUESTED && COMPATIBLE && operational;
    }

    public static void onPipelineReset() {
        CANDIDATES.clear();
    }

    public static void buildPlan(
        Map<String, ?> variables,
        Map<String, ?> expressions,
        Map<?, ?> dependsOn,
        List<?> uniformOrder
    ) {
        CANDIDATES.clear();

        if (!isEnabled()) {
            return;
        }

        try {
            ReflectionModel model = reflectionModel();
            IdentityHashMap<Object, Boolean> activeUniforms = new IdentityHashMap<>();
            for (Object uniform : uniformOrder) {
                activeUniforms.put(uniform, Boolean.TRUE);
            }

            int pure = 0;
            int stateful = 0;
            int nondeterministic = 0;
            int unknown = 0;

            for (Map.Entry<String, ?> entry : variables.entrySet()) {
                Object uniform = entry.getValue();

                if (!activeUniforms.containsKey(uniform)) {
                    continue;
                }

                Object expression = expressions.get(entry.getKey());
                Classification classification = classify(expression, model);

                switch (classification) {
                    case PURE -> {
                        List<?> dependencies = (List<?>) dependsOn.get(uniform);
                        Object[] dependencyArray =
                            dependencies == null ? new Object[0] : dependencies.toArray();

                        CANDIDATES.put(uniform, new CandidateState(dependencyArray));
                        pure++;
                    }
                    case STATEFUL -> stateful++;
                    case NONDETERMINISTIC -> nondeterministic++;
                    case UNKNOWN -> unknown++;
                }
            }

            Argon.LOGGER.info(
                "[Phase B][Iris uniforms] Evaluation plan: pureCandidates={}, stateful={}, nondeterministic={}, unknown={}.",
                pure,
                stateful,
                nondeterministic,
                unknown
            );
        } catch (ReflectiveOperationException | RuntimeException e) {
            disableForSession("Could not inspect Iris/Stareval expression structure.", e);
        }
    }

    public static boolean shouldSkip(Object uniform) {
        if (!isEnabled()) {
            return false;
        }

        CandidateState state = CANDIDATES.get(uniform);
        if (state == null || !state.initialized) {
            return false;
        }

        Object[] dependencies = state.dependencies;

        for (int i = 0; i < dependencies.length; i++) {
            Object dependency = dependencies[i];

            if (!(dependency instanceof IrisUniformDeduplicator.UniformState revisionState)) {
                disableForSession(
                    "A dependency did not expose Argon's revision state.",
                    null
                );
                return false;
            }

            if (revisionState.argon$revision() != state.dependencyRevisions[i]) {
                return false;
            }
        }

        IrisUniformInstrumentation.onPhaseBEvaluationSkip();
        return true;
    }

    public static void onEvaluated(Object uniform) {
        if (!isEnabled()) {
            return;
        }

        CandidateState state = CANDIDATES.get(uniform);
        if (state == null) {
            return;
        }

        Object[] dependencies = state.dependencies;

        for (int i = 0; i < dependencies.length; i++) {
            Object dependency = dependencies[i];

            if (!(dependency instanceof IrisUniformDeduplicator.UniformState revisionState)) {
                disableForSession(
                    "A dependency did not expose Argon's revision state.",
                    null
                );
                return;
            }

            state.dependencyRevisions[i] = revisionState.argon$revision();
        }

        state.initialized = true;
        IrisUniformInstrumentation.onPhaseBCandidateEvaluation();
    }

    private static Classification classify(Object expression, ReflectionModel model)
        throws ReflectiveOperationException {
        if (expression == null) {
            return Classification.UNKNOWN;
        }

        if (model.constantExpressionClass.isInstance(expression)) {
            return Classification.PURE;
        }

        if (model.callExpressionClass.isInstance(expression)) {
            Object function = model.callFunction.get(expression);
            boolean pure = (boolean) model.functionIsPure.invoke(function);

            if (!pure) {
                return Classification.STATEFUL;
            }

            String functionName = model.staticFunctionNames.get(function);

            if ("random".equals(functionName) || "randomInt".equals(functionName)) {
                return Classification.NONDETERMINISTIC;
            }

            if (functionName == null) {
                // Dynamic functions are conservatively unknown unless Iris marks
                // them impure above (e.g. smooth).
                return Classification.UNKNOWN;
            }

            Object[] arguments = (Object[]) model.callArguments.get(expression);
            Classification combined = Classification.PURE;

            for (Object argument : arguments) {
                combined = Classification.combine(combined, classify(argument, model));
                if (combined != Classification.PURE) {
                    return combined;
                }
            }

            return combined;
        }

        if (model.variableExpressionClass.isInstance(expression)) {
            // Variable safety is determined from the revision-tracked dependency
            // graph built by Iris, not from update-frequency labels.
            return Classification.PURE;
        }

        return Classification.UNKNOWN;
    }

    private static ReflectionModel reflectionModel() throws ReflectiveOperationException {
        if (reflection != null) {
            return reflection;
        }

        ClassLoader loader = IrisUniformEvaluationPlanner.class.getClassLoader();

        Class<?> callExpressionClass =
            Class.forName("kroppeb.stareval.expression.CallExpression", false, loader);
        Class<?> constantExpressionClass =
            Class.forName("kroppeb.stareval.expression.ConstantExpression", false, loader);
        Class<?> variableExpressionClass =
            Class.forName("kroppeb.stareval.expression.VariableExpression", false, loader);
        Class<?> typedFunctionClass =
            Class.forName("kroppeb.stareval.function.TypedFunction", false, loader);
        Class<?> irisFunctionsClass =
            Class.forName("net.irisshaders.iris.parsing.IrisFunctions", false, loader);

        Field callFunction = callExpressionClass.getDeclaredField("function");
        Field callArguments = callExpressionClass.getDeclaredField("arguments");
        callFunction.setAccessible(true);
        callArguments.setAccessible(true);

        Method functionIsPure = typedFunctionClass.getMethod("isPure");

        Field resolverField = irisFunctionsClass.getField("functions");
        Object resolver = resolverField.get(null);

        Field staticFunctionsField = resolver.getClass().getDeclaredField("functions");
        staticFunctionsField.setAccessible(true);

        IdentityHashMap<Object, String> staticFunctionNames = new IdentityHashMap<>();

        Map<?, ?> functionsByName = (Map<?, ?>) staticFunctionsField.get(resolver);
        for (Map.Entry<?, ?> nameEntry : functionsByName.entrySet()) {
            String name = String.valueOf(nameEntry.getKey());
            Map<?, ?> byReturnType = (Map<?, ?>) nameEntry.getValue();

            for (Object overloadsValue : byReturnType.values()) {
                for (Object function : (List<?>) overloadsValue) {
                    staticFunctionNames.put(function, name);
                }
            }
        }

        reflection = new ReflectionModel(
            callExpressionClass,
            constantExpressionClass,
            variableExpressionClass,
            callFunction,
            callArguments,
            functionIsPure,
            staticFunctionNames
        );

        return reflection;
    }

    private static void disableForSession(String reason, Throwable cause) {
        if (!operational) {
            return;
        }

        operational = false;
        CANDIDATES.clear();

        if (cause == null) {
            Argon.LOGGER.error(
                "Disabling experimental Iris uniform evaluation caching for this session: {}",
                reason
            );
        } else {
            Argon.LOGGER.error(
                "Disabling experimental Iris uniform evaluation caching for this session: {}",
                reason,
                cause
            );
        }
    }

    private enum Classification {
        PURE,
        STATEFUL,
        NONDETERMINISTIC,
        UNKNOWN;

        private static Classification combine(Classification left, Classification right) {
            if (left == STATEFUL || right == STATEFUL) {
                return STATEFUL;
            }

            if (left == NONDETERMINISTIC || right == NONDETERMINISTIC) {
                return NONDETERMINISTIC;
            }

            if (left == UNKNOWN || right == UNKNOWN) {
                return UNKNOWN;
            }

            return PURE;
        }
    }

    private static final class CandidateState {
        private final Object[] dependencies;
        private final long[] dependencyRevisions;
        private boolean initialized;

        private CandidateState(Object[] dependencies) {
            this.dependencies = dependencies;
            this.dependencyRevisions = new long[dependencies.length];
        }
    }

    private record ReflectionModel(
        Class<?> callExpressionClass,
        Class<?> constantExpressionClass,
        Class<?> variableExpressionClass,
        Field callFunction,
        Field callArguments,
        Method functionIsPure,
        IdentityHashMap<Object, String> staticFunctionNames
    ) {
    }
}
