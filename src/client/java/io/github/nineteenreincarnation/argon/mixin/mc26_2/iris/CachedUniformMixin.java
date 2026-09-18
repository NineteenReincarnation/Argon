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

@Pseudo
@Mixin(targets = "net.irisshaders.iris.uniforms.custom.cached.CachedUniform", remap = false)
abstract class CachedUniformMixin implements IrisUniformDeduplicator.UniformState {
    @Shadow(remap = false)
    private boolean changed;

    @Shadow(remap = false)
    public abstract void push(int location);

    @Unique
    private long argon$revision;

    @Override
    @Unique
    public long argon$revision() {
        return argon$revision;
    }

    @Override
    @Unique
    public void argon$incrementRevision() {
        argon$revision++;
        IrisUniformDeduplicator.onUniformChanged();
    }

    @Override
    @Unique
    public void argon$push(int location) {
        push(location);
    }

    @Inject(method = "update", at = @At("HEAD"), remap = false)
    private void argon$countEvaluation(CallbackInfo ci) {
        IrisUniformInstrumentation.onEvaluation();
    }

    @Inject(method = "pushIfChanged", at = @At("HEAD"), remap = false)
    private void argon$countUploadCheck(int location, CallbackInfo ci) {
        IrisUniformInstrumentation.onUploadCheck(changed);
    }
}
