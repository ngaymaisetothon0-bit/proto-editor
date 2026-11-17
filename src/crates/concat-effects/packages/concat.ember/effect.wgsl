struct Params { tone: f32, contrast: f32 }

// Firelight: warm midtones, a little more saturation, the highlights held.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.16, 0.04, -0.06), tone);
    out = saturation(out, 1.0 + tone * 0.25);
    out = split_tone(out, vec3<f32>(0.08, 0.01, -0.04), vec3<f32>(0.02, 0.0, -0.01), tone);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
