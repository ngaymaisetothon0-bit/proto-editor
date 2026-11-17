struct Params { tone: f32, juice: f32 }

// Bright tangerine colour: juicy orange warmth, sunny and bold.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.0, -0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.22);
    out = saturation(out, 1.0 + tone * 0.1);
    out = white_balance(out, 6500.0 - params.juice * 12.0);
    return vec4<f32>(out, c.a);
}
