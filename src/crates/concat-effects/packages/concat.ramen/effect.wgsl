struct Params { tone: f32, steam: f32 }

// Steaming hot ramen: rich warm browns, soft highlights, straight off the stove.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.01, -0.02), tone);
    out = saturation(out, 1.0 + tone * 0.2);
    out = white_balance(out, 6500.0 - params.steam * 20.0);
    out = matte(out, tone * 0.04, 1.0);
    return vec4<f32>(out, c.a);
}
