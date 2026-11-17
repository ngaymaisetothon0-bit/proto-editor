struct Params { duration: f32 }

// The clip blooms in, its brightness rising from dark to full over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let out = sample(uv);
    let bright = 0.2 + 0.8 * p;
    let col = out.rgb * bright;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
