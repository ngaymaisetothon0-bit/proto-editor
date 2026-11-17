struct Params { tone: f32, depth: f32 }

// Deep teal water: cool blue-green colour, calm and clear.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.02, 0.02, 0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = saturation(out, 1.0 + tone * 0.1);
    out = white_balance(out, 6500.0 + params.depth * 10.0);
    return vec4<f32>(out, c.a);
}
