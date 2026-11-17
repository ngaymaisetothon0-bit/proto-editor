struct Params { tone: f32, contrast: f32 }

// An aged print: the picture fades toward a sepia tone, darker in the shadows.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let l = luma(c.rgb);
    let sep = vec3<f32>(l * 1.07, l * 0.74, l * 0.43);
    var out = mix(c.rgb, sep, tone);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
