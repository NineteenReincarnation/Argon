package io.github.nineteenreincarnation.argon.mixin.iris;

import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.custom.cached.CachedUniform", remap = false)
abstract class CachedUniformMixin {
    @Shadow(remap = false)
    private boolean changed;

    @Inject(method = "update", at = @At("HEAD"), remap = false)
    private void argon$countEvaluation(CallbackInfo ci) {
        IrisUniformInstrumentation.onEvaluation();
    }

    @Inject(method = "pushIfChanged", at = @At("HEAD"), remap = false)
    private void argon$countUploadCheck(int location, CallbackInfo ci) {
        IrisUniformInstrumentation.onUploadCheck(changed);
    }
}
