package io.github.nineteenreincarnation.argon.mixin.iris;

import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Pseudo
@Mixin(targets = "net.irisshaders.iris.pipeline.IrisRenderingPipeline", remap = false)
abstract class IrisRenderingPipelineMixin {
    @Inject(method = "beginLevelRendering", at = @At("HEAD"), remap = false)
    private void argon$beginUniformFrame(CallbackInfo ci) {
        IrisUniformInstrumentation.onFrameStart();
    }
}
