struct Params { tone: f32, pastel: f32 }

// Pink shadows and cyan highlights, muted to pastel, blacks lifted.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.05, -0.01, 0.02), vec3<f32>(-0.02, 0.01, 0.04), tone);
    out = saturation(out, 1.0 - params.pastel / 100.0 * 0.25);
    out = matte(out, tone * 0.08, 1.0);
    return vec4<f32>(out, c.a);
}
