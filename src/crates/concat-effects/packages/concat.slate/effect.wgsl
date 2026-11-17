struct Params { tone: f32, lift: f32 }

// Cool slate grey: black and white with a blue-grey soul, lifted blacks, quiet and strong.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.25, 0.35, 0.40));
    var out = matte(g, params.lift / 100.0 * 0.12, 1.0);
    out = contrast(out, 1.0 - tone * 0.05);
    out = tint_midtones(out, vec3<f32>(-0.01, 0.0, 0.02), tone);
    return vec4<f32>(out, c.a);
}
