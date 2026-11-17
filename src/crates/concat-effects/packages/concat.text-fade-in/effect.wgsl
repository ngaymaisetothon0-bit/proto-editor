struct Params { duration: f32 }

// The whole title fades in softly over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    let c = sample(uv);
    return vec4<f32>(c.rgb * p, c.a * p);
}
