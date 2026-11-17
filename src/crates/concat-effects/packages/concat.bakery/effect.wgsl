struct Params { tone: f32, warmth: f32 }

// Fresh from the oven: golden browns, warm light, bread that looks baked.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.025, -0.03), tone);
    out = white_balance(out, 6500.0 - params.warmth * 28.0);
    out = saturation(out, 1.0 + tone * 0.18);
    out = contrast(out, 1.0 + tone * 0.08);
    return vec4<f32>(out, c.a);
}
