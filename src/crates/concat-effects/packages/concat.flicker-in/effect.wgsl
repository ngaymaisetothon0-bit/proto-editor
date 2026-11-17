struct Params { duration: f32 }

// The clip flickers in, its brightness fluttering before it settles over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let t = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let out = sample(uv);
    // Exactly 1.0 at t = 1, so the clip settles at its true brightness.
    let flick = 1.0 - 0.4 * sin(t * 18.8495559) * (1.0 - t);
    let col = out.rgb * flick;
    let fade = smoothstep(0.0, 1.0, t);
    return vec4<f32>(col * fade, out.a * fade);
}
