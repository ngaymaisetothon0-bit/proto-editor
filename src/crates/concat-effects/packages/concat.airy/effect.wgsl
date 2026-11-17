struct Params { tone: f32, contrast: f32 }

// Bright and soft: shadows lifted, contrast eased, a breath of cool air.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.10);
    out = contrast(out, 1.0 - params.contrast / 100.0 * 0.35);
    out = saturation(out, 1.0 - tone * 0.15);
    out = white_balance(out, 6500.0 + tone * 800.0);
    return vec4<f32>(out, c.a);
}
