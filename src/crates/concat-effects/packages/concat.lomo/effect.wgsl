struct Params { tone: f32, contrast: f32 }

// The toy camera: saturated, punchy, corners pulled dark.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.45);
    out = s_curve(out, params.contrast / 100.0 * 0.6 + tone * 0.2);
    out = split_tone(out, vec3<f32>(0.04, 0.02, -0.02), vec3<f32>(0.02, 0.0, -0.03), tone);
    out = vignette(out, uv, tone * 0.55);
    return vec4<f32>(out, c.a);
}
