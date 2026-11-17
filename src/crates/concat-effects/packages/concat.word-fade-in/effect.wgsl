struct Params { duration: f32 }

// Each word fades in softly, one after another, in reading order.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    let c = sample(uv);
    return vec4<f32>(c.rgb * w, c.a * w);
}
