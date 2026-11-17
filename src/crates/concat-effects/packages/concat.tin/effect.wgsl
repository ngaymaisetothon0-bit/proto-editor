struct Params { tone: f32, roof: f32 }

// Tin, dull and grey: black and white, soft and flat, like old roof.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.32, 0.37));
    var out = fade(g, params.roof / 100.0 * 0.04);
    out = contrast(out, 1.0 + tone * 0.09);
    return vec4<f32>(out, c.a);
}
