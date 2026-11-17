struct Params { tone: f32, contrast: f32 }

// Dusk: violet in the shadows, amber in the highlights, the midtones held back.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 - tone * 0.25);
    out = split_tone(out, vec3<f32>(0.06, 0.0, 0.10), vec3<f32>(0.10, 0.04, -0.05), tone);
    out = tint_midtones(out, vec3<f32>(0.02, -0.01, 0.04), tone);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
