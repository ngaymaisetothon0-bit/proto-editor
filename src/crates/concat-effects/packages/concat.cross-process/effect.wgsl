struct Params { tone: f32, contrast: f32 }

// Cross-processed film: punchy contrast, warm highlights, yellow-green blacks.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.20);
    out = split_tone(out, vec3<f32>(0.04, 0.07, -0.03), vec3<f32>(0.09, 0.03, -0.06), tone);
    out = fade(out, tone * 0.06);
    out = s_curve(out, params.contrast / 100.0 * 0.8);
    return vec4<f32>(out, c.a);
}
