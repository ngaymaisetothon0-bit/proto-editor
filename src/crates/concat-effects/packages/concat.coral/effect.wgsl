struct Params { tone: f32, reef: f32 }

// Warm coral colour: pink-orange life in the midtones, the picture glowing like a reef at noon.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.04, -0.01, -0.01), tone);
    out = vibrance(out, 1.0 + tone * 0.22);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 - params.reef * 10.0);
    return vec4<f32>(out, c.a);
}
