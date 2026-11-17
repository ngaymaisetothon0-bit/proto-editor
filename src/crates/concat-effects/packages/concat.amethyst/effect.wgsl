struct Params { tone: f32, gem: f32 }

// Amethyst stone: violet, calm and precious, a gem colour.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.01, -0.02, 0.03), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 - params.gem * 8.0);
    return vec4<f32>(out, c.a);
}
