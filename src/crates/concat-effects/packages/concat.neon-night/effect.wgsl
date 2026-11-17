struct Params { tone: f32, neon: f32 }

// City at night in neon: violet-blue shadows, pink highlights, electric glow.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.02, -0.02, 0.05), vec3<f32>(0.04, -0.01, -0.02), tone);
    out = saturation(out, 1.0 + tone * 0.2);
    out = contrast(out, 1.0 + tone * 0.1);
    return vec4<f32>(out, c.a);
}
