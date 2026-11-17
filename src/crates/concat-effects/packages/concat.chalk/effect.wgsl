struct Params { tone: f32, dust: f32 }

// Chalk on a blackboard: black and white, soft and dusty, like a school drawing.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.33, 0.37));
    var out = fade(g, params.dust / 100.0 * 0.06);
    out = contrast(out, 1.0 + tone * 0.08);
    return vec4<f32>(out, c.a);
}
