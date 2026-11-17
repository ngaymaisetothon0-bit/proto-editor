struct Params { tone: f32, punch: f32 }

// Old Hollywood in full colour: bold saturation, punchy contrast, warm glow.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.45);
    out = white_balance(out, 6500.0 - params.punch * 12.0);
    out = s_curve(out, tone * 0.35);
    out = contrast(out, 1.0 + tone * 0.12);
    return vec4<f32>(out, c.a);
}
