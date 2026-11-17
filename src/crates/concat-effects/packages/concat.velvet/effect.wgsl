struct Params { tone: f32, soft: f32 }

// Soft glamour portrait: warm skin, lifted blacks, gentle contrast.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.04, 0.01, -0.02), tone);
    out = saturation(out, 1.0 + tone * 0.12);
    out = matte(out, tone * 0.06, 1.0 - tone * 0.03);
    out = contrast(out, 1.0 - params.soft / 100.0 * 0.12);
    return vec4<f32>(out, c.a);
}
