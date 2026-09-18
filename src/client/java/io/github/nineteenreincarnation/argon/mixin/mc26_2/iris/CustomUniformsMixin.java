package io.github.nineteenreincarnation.argon.mixin.mc26_2.iris;

import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformDeduplicator;
import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.custom.CustomUniforms", remap = false)
abstract class CustomUniformsMixin {
    @Shadow(remap = false)
    private Map<Object, ?> locationMap;

    @Unique
    private long argon$updateStartedNanos;

    @Unique
    private long argon$pushStartedNanos;

    @Inject(method = "optimise", at = @At("HEAD"), remap = false)
    private void argon$pipelineReady(CallbackInfo ci) {
        IrisUniformDeduplicator.onPipelineReset();
        IrisUniformInstrumentation.onPipelineReset();
    }

    @Inject(method = "update", at = @At("HEAD"), remap = false)
    private void argon$beginUpdate(CallbackInfo ci) {
        IrisUniformDeduplicator.onUniformUpdateStart();

        if (IrisUniformInstrumentation.isMeasuring()) {
            argon$updateStartedNanos = System.nanoTime();
        }
    }

    @Inject(method = "update", at = @At("RETURN"), remap = false)
    private void argon$endUpdate(CallbackInfo ci) {
        IrisUniformDeduplicator.onUniformUpdateEnd();

        if (IrisUniformInstrumentation.isMeasuring()) {
            IrisUniformInstrumentation.onUpdateDuration(System.nanoTime() - argon$updateStartedNanos);
        }
    }

    @Inject(method = "push", at = @At("HEAD"), cancellable = true, remap = false)
    private void argon$beginPush(Object pass, CallbackInfo ci) {
        Object mappedUniforms = locationMap.get(pass);
        IrisUniformInstrumentation.onPassPush(pass, mappedUniforms);

        boolean measuring = IrisUniformInstrumentation.isMeasuring();
        if (measuring) {
            argon$pushStartedNanos = System.nanoTime();
        }

        if (IrisUniformDeduplicator.tryPush(pass, mappedUniforms)) {
            if (measuring) {
                IrisUniformInstrumentation.onPushDuration(System.nanoTime() - argon$pushStartedNanos);
            }

            ci.cancel();
        }
    }

    @Inject(method = "push", at = @At("RETURN"), remap = false)
    private void argon$endPush(Object pass, CallbackInfo ci) {
        if (IrisUniformInstrumentation.isMeasuring() && !IrisUniformDeduplicator.isEnabled()) {
            IrisUniformInstrumentation.onPushDuration(System.nanoTime() - argon$pushStartedNanos);
        }
    }
}
