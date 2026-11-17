struct Params { tone: f32, dream: f32 }

// Lavender dream: violet shadows, soft pink light, gently muted.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.03, -0.01, 0.04), vec3<f32>(-0.01, -0.01, 0.03), tone);
    out = saturation(out, 1.0 + tone * 0.1);
    out = matte(out, tone * 0.05, 1.0);
    return vec4<f32>(out, c.a);
}
