struct Params { tone: f32, deep: f32 }

// Deep black ink: black and white with bite, dark blacks, crisp whites, like a fresh print.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.35, 0.35));
    var out = matte(g, params.deep / 100.0 * 0.05, 1.0);
    out = contrast(out, 1.0 + tone * 0.12);
    return vec4<f32>(out, c.a);
}
