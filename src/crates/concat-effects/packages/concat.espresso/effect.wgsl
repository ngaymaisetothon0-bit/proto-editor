struct Params { tone: f32, warmth: f32 }

// Cafe dark: warm brown midtones, deep contrast, food that looks roasted.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.02, -0.04), tone);
    out = white_balance(out, 6500.0 - params.warmth * 25.0);
    out = saturation(out, 1.0 + tone * 0.15);
    out = contrast(out, 1.0 + tone * 0.12);
    return vec4<f32>(out, c.a);
}
