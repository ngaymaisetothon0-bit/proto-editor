struct Params { tone: f32, steel: f32 }

// Cold zinc grey: black and white with a metallic edge, sharp and clean.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.35, 0.35));
    var out = matte(g, params.steel / 100.0 * 0.04, 1.0);
    out = contrast(out, 1.0 + tone * 0.11);
    return vec4<f32>(out, c.a);
}
