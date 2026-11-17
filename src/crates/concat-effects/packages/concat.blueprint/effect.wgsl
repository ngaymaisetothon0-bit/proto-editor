struct Params { tone: f32, contrast: f32 }

// Cyanotype: the picture redrawn as a blueprint, deep blue over pale blue-white.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let l = luma(c.rgb);
    var ramp = vec3<f32>(l * 0.10, l * 0.32, l * 0.85);
    ramp = s_curve(ramp, params.contrast / 100.0 * 0.6);
    var out = mix(c.rgb, ramp, tone);
    return vec4<f32>(out, c.a);
}
