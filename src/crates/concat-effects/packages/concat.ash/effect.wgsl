struct Params { tone: f32, soft: f32 }

// Soft ash grey: black and white, gentle and even, no harsh blacks, no blown whites.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.33, 0.33, 0.34));
    var out = matte(g, params.soft / 100.0 * 0.15, 1.0);
    out = contrast(out, 1.0 - tone * 0.08);
    return vec4<f32>(out, c.a);
}
