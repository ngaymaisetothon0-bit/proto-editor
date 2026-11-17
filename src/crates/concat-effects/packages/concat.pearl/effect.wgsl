struct Params { tone: f32, glow: f32 }

// Pearl skin: bright, luminous, a cool white glow that softens everything.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.01, -0.005, 0.02), tone);
    out = white_balance(out, 6500.0 + params.glow * 12.0);
    out = matte(out, tone * 0.06, 1.0);
    out = contrast(out, 1.0 - tone * 0.08);
    return vec4<f32>(out, c.a);
}
