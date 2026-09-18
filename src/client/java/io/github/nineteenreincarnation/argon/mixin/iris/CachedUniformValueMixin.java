package io.github.nineteenreincarnation.argon.mixin.iris;

import io.github.nineteenreincarnation.argon.client.compat.iris.IrisUniformInstrumentation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Pseudo
@Mixin(
    targets = {
        "net.irisshaders.iris.uniforms.custom.cached.FloatCachedUniform",
        "net.irisshaders.iris.uniforms.custom.cached.IntCachedUniform",
        "net.irisshaders.iris.uniforms.custom.cached.BooleanCachedUniform",
        "net.irisshaders.iris.uniforms.custom.cached.VectorCachedUniform"
    },
    remap = false
)
abstract class CachedUniformValueMixin {
    @Inject(method = "doUpdate", at = @At("RETURN"), remap = false)
    private void argon$countValueChange(CallbackInfoReturnable<Boolean> cir) {
        IrisUniformInstrumentation.onEvaluationResult(cir.getReturnValueZ());
    }
}
