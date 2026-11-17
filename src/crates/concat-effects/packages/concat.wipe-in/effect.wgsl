struct Params { duration: f32 }

// The clip wipes in from the left, revealing itself edge first over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    if (uv.x > p) {
        return vec4<f32>(0.0);
    }
    return sample(uv);
}
