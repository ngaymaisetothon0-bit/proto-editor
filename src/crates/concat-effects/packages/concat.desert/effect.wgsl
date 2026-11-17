struct Params { tone: f32, heat: f32 }

// Sun-bleached sand: warm, dry, the horizon glowing with heat.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.06, 0.02, -0.05), tone);
    out = white_balance(out, 6500.0 - params.heat * 30.0);
    out = vibrance(out, 1.0 + tone * 0.15);
    out = matte(out, tone * 0.04, 1.0 - tone * 0.03);
    return vec4<f32>(out, c.a);
}
