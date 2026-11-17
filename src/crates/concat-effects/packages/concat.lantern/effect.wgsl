struct Params { tone: f32, glow: f32 }

// Warm lantern glow: golden light in the dark, soft and inviting.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.02, 0.0, -0.02), vec3<f32>(0.02, 0.0, -0.01), tone);
    out = white_balance(out, 6500.0 - params.glow * 18.0);
    out = fade(out, tone * 0.04);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
