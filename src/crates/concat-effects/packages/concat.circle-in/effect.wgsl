struct Params { duration: f32 }

// The clip opens out of a circle from the centre, like an iris, over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    // Distance from the centre in pixels; the ring grows to 0.75 of the
    // frame diagonal, which covers the far corner.
    let d = distance((uv - vec2<f32>(0.5)) * frame.size, vec2<f32>(0.0));
    let r = p * 0.75 * length(frame.size);
    if (d > r) {
        return vec4<f32>(0.0);
    }
    return sample(uv);
}
