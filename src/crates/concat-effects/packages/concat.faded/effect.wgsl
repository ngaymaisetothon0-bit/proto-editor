struct Params { tone: f32, contrast: f32 }

// A print left in the sun: blacks lifted, contrast eased, a warm cast.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = matte(c.rgb, tone * 0.12, 1.0 - tone * 0.05);
    out = saturation(out, 1.0 - tone * 0.2);
    out = tint_midtones(out, vec3<f32>(0.03, 0.01, -0.02), tone);
    out = contrast(out, 1.0 - params.contrast / 100.0 * 0.3);
    return vec4<f32>(out, c.a);
}
