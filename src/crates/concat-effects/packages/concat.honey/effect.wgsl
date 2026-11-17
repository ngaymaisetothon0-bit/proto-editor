struct Params { tone: f32, glow: f32 }

// Honey light on skin: warm midtones, lifted blacks, soft contrast.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.04, 0.015, -0.03), tone);
    out = saturation(out, 1.0 + tone * 0.1);
    out = matte(out, tone * 0.03, 1.0 - tone * 0.02);
    out = contrast(out, 1.0 - params.glow / 100.0 * 0.1);
    return vec4<f32>(out, c.a);
}
