struct Params { tone: f32, sunny: f32 }

// A sunny picnic afternoon: warm, cheerful, a little extra life in the greens.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.04, 0.02, -0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = white_balance(out, 6500.0 - params.sunny * 15.0);
    out = contrast(out, 1.0 + tone * 0.06);
    return vec4<f32>(out, c.a);
}
