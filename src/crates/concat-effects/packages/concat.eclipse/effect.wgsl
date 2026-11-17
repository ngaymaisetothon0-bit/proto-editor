struct Params { tone: f32, shadow: f32 }

// An eclipse: the light gone dark at the edges, colour muted, a ring of drama around the frame.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(-0.02, -0.01, 0.02), vec3<f32>(0.01, 0.0, -0.01), tone);
    out = saturation(out, 1.0 - tone * 0.15);
    out = contrast(out, 1.0 + tone * 0.12);
    out = vignette(out, uv, tone * 0.45);
    return vec4<f32>(out, c.a);
}
