struct Params { duration: f32 }

// The title types itself in one word at a time, each word fading up as its turn comes.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let a = clamp((p - reveal_order(uv) * 0.75) * 8.0, 0.0, 1.0);
    let c = sample(uv);
    return vec4<f32>(c.rgb * a, c.a * a);
}
