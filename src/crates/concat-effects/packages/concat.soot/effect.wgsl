struct Params { tone: f32, smoke: f32 }

// Soot on the wall: black and white, dark and smoky, like a chimney sweep's sketch.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.32, 0.34, 0.34));
    var out = matte(g, params.smoke / 100.0 * 0.05, 1.0);
    out = contrast(out, 1.0 + tone * 0.1);
    return vec4<f32>(out, c.a);
}
