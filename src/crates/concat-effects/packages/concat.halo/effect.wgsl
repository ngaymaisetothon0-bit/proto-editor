struct Params { tone: f32, glow: f32 }

// Soft light all around: lifted blacks, gentle contrast, a bright halo over everything.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.08);
    out = contrast(out, 1.0 - tone * 0.10);
    out = saturation(out, 1.0 - tone * 0.10);
    out = white_balance(out, 6500.0 + params.glow * 10.0);
    return vec4<f32>(out, c.a);
}
