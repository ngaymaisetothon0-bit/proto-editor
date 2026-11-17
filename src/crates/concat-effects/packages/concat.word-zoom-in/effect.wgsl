struct Params { duration: f32 }

// Each word settles into place from larger and fainter, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // 150% of size and fully transparent before the word's turn.
    let scale = 1.5 - 0.5 * w;
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    let out = sample(coord);
    return vec4<f32>(out.rgb * w, out.a * w);
}
