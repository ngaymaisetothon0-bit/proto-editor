struct Params { tone: f32, warmth: f32 }

// Golden hour bottled: orange shadows, pink highlights, warm and soft.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.05, 0.01, -0.03), vec3<f32>(0.03, -0.01, -0.02), tone);
    out = white_balance(out, 6500.0 - params.warmth * 30.0);
    out = saturation(out, 1.0 + tone * 0.2);
    out = matte(out, tone * 0.04, 1.0);
    return vec4<f32>(out, c.a);
}
