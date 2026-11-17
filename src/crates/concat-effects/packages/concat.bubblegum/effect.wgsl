struct Params { tone: f32, sweet: f32 }

// Sweet bubblegum pink: playful, bright, a little candy-coloured daydream.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.03, -0.01, 0.01), vec3<f32>(0.02, -0.02, 0.01), tone);
    out = saturation(out, 1.0 + tone * 0.2);
    out = contrast(out, 1.0 + tone * 0.04);
    return vec4<f32>(out, c.a);
}
