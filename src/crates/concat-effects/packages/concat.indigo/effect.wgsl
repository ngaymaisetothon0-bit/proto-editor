struct Params { tone: f32, dye: f32 }

// Deep indigo dye: rich blue-violet colour, dark and royal.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.01, -0.01, 0.04), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 + params.dye * 10.0);
    return vec4<f32>(out, c.a);
}
