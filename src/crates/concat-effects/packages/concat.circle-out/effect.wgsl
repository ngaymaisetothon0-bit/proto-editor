struct Params { duration: f32 }

// The clip closes away into a circle at the centre, like an iris shutting, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    // Distance from the centre in pixels; the ring shrinks from 0.75 of
    // the frame diagonal, which covers the far corner.
    let d = distance((uv - vec2<f32>(0.5)) * frame.size, vec2<f32>(0.0));
    let r = p * 0.75 * length(frame.size);
    if (d > r) {
        return vec4<f32>(0.0);
    }
    return sample(uv);
}
