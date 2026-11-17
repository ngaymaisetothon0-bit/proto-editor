struct Params { tone: f32, lift: f32 }

// Soft black and white: lifted blacks, gentle contrast, no harsh clip.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.33, 0.37));
    var out = matte(g, params.lift / 100.0 * 0.15, 1.0);
    out = contrast(out, 1.0 - tone * 0.12);
    return vec4<f32>(out, c.a);
}
