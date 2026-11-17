struct Params { tone: f32, morning: f32 }

// Morning in the courtyard: plain, warm, quiet — an ordinary good day.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.04);
    out = white_balance(out, 6500.0 - params.morning * 8.0);
    out = vibrance(out, 1.0 + tone * 0.13);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
