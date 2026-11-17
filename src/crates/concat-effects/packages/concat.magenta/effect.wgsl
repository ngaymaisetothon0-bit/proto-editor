struct Params { tone: f32, vivid: f32 }

// Bold magenta colour: vivid pink-purple energy, loud and proud.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.03, -0.02, 0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.22);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 + params.vivid * 8.0);
    return vec4<f32>(out, c.a);
}
