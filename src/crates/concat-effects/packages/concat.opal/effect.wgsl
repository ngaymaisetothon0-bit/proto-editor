struct Params { tone: f32, gem: f32 }

// Opal stone: soft, cool, milky — a gem colour that changes with the light.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.01, 0.0, 0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 + params.gem * 6.0);
    return vec4<f32>(out, c.a);
}
