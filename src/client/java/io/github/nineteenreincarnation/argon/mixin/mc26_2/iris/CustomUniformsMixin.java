package io.github.nineteenreincarnation.argon.mixin.mc26_2.iris;

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

    @Inject(method = "<init>", at = @At("RETURN"), remap = false)
    private void argon$pipelineCreated(CallbackInfo ci) {
        IrisUniformInstrumentation.onPipelineReset();
    }

    @Inject(method = "update", at = @At("HEAD"), remap = false)
    private void argon$beginUpdate(CallbackInfo ci) {
        if (IrisUniformInstrumentation.isMeasuring()) {
            argon$updateStartedNanos = System.nanoTime();
        }
    }

    @Inject(method = "update", at = @At("RETURN"), remap = false)
    private void argon$endUpdate(CallbackInfo ci) {
        if (IrisUniformInstrumentation.isMeasuring()) {
            IrisUniformInstrumentation.onUpdateDuration(System.nanoTime() - argon$updateStartedNanos);
        }
    }

    @Inject(method = "push", at = @At("HEAD"), remap = false)
    private void argon$beginPush(Object pass, CallbackInfo ci) {
        IrisUniformInstrumentation.onPassPush(pass, locationMap.get(pass));

        if (IrisUniformInstrumentation.isMeasuring()) {
            argon$pushStartedNanos = System.nanoTime();
        }
    }

    @Inject(method = "push", at = @At("RETURN"), remap = false)
    private void argon$endPush(Object pass, CallbackInfo ci) {
        if (IrisUniformInstrumentation.isMeasuring()) {
            IrisUniformInstrumentation.onPushDuration(System.nanoTime() - argon$pushStartedNanos);
        }
    }
}
