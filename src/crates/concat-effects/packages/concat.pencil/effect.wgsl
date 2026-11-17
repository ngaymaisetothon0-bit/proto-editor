struct Params { tone: f32, sketch: f32 }

// A pencil sketch: black and white, soft grey, crisp lines — drawn by hand.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.33, 0.33, 0.34));
    var out = matte(g, params.sketch / 100.0 * 0.04, 1.0);
    out = contrast(out, 1.0 + tone * 0.1);
    return vec4<f32>(out, c.a);
}
