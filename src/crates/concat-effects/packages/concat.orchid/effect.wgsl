struct Params { tone: f32, bloom: f32 }

// An orchid in bloom: soft violet, rich and delicate, a flower colour.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.02, -0.02, 0.03), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = saturation(out, 1.0 + tone * 0.12);
    out = white_balance(out, 6500.0 - params.bloom * 8.0);
    return vec4<f32>(out, c.a);
}
