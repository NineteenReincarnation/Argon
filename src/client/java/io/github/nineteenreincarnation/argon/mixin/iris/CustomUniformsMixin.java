package io.github.nineteenreincarnation.argon.mixin.iris;

import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.custom.CustomUniforms", remap = false)
abstract class CustomUniformsMixin {
    @Unique
    private long argon$updateStartedNanos;

    @Unique
    private long argon$pushStartedNanos;

    @Inject(method = "update", at = @At("HEAD"), remap = false)
    private void argon$beginUpdate(CallbackInfo ci) {
        if (IrisUniformInstrumentation.isEnabled()) {
            argon$updateStartedNanos = System.nanoTime();
        }
    }

    @Inject(method = "update", at = @At("RETURN"), remap = false)
    private void argon$endUpdate(CallbackInfo ci) {
        if (IrisUniformInstrumentation.isEnabled()) {
            IrisUniformInstrumentation.onUpdateDuration(System.nanoTime() - argon$updateStartedNanos);
        }
    }

    @Inject(method = "push", at = @At("HEAD"), remap = false)
    private void argon$beginPush(Object pass, CallbackInfo ci) {
        if (IrisUniformInstrumentation.isEnabled()) {
            IrisUniformInstrumentation.onPassPush();
            argon$pushStartedNanos = System.nanoTime();
        }
    }

    @Inject(method = "push", at = @At("RETURN"), remap = false)
    private void argon$endPush(Object pass, CallbackInfo ci) {
        if (IrisUniformInstrumentation.isEnabled()) {
            IrisUniformInstrumentation.onPushDuration(System.nanoTime() - argon$pushStartedNanos);
        }
    }
}
