struct Params { duration: f32 }

// Each word rises into place from below and fades in, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    let shift = (1.0 - w) * 0.2;
    let coord = uv - vec2<f32>(0.0, shift);
    // The sampler clamps to the edge, so a coord outside the picture
    // would smear its edge instead of showing nothing: say so plainly.
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * w, c.a * w);
}
