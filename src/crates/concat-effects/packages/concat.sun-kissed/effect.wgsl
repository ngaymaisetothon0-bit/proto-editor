struct Params { tone: f32, tan: f32 }

// A day in the sun on skin: warm tan, gentle saturation, soft glow.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.045, 0.015, -0.035), tone);
    out = saturation(out, 1.0 + tone * 0.15);
    out = white_balance(out, 6500.0 - params.tan * 25.0);
    out = contrast(out, 1.0 + tone * 0.06);
    return vec4<f32>(out, c.a);
}
