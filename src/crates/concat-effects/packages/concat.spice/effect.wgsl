struct Params { tone: f32, heat: f32 }

// Rich and spicy: deep reds and warm oranges that make food look irresistible.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.06, -0.01, -0.03), tone);
    out = saturation(out, 1.0 + tone * 0.25);
    out = white_balance(out, 6500.0 - params.heat * 22.0);
    out = s_curve(out, tone * 0.2);
    return vec4<f32>(out, c.a);
}
