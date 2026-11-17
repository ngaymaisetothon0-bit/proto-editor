struct Params { duration: f32 }

// The clip shimmers in, its brightness rippling up and down before it settles over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let p = 1.0 - pow(1.0 - t, 3.0);
    let out = sample(uv);
    let shimmer = 1.0 + 0.25 * sin(t * 12.5663706) * (1.0 - t);
    let col = out.rgb * shimmer;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
