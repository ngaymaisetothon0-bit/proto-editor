struct Params { tone: f32, contrast: f32 }

// Two inks: teal in the shadows, warm cream in the highlights.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let l = luma(c.rgb);
    let duo = mix(vec3<f32>(0.05, 0.16, 0.20), vec3<f32>(0.98, 0.90, 0.72), l);
    var out = mix(c.rgb, duo, tone);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
