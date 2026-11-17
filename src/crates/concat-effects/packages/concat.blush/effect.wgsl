struct Params { tone: f32, rose: f32 }

// A soft blush on the cheeks: gentle pink warmth, flattering and kind.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.03, -0.01, 0.0), tone);
    out = saturation(out, 1.0 + tone * 0.1);
    out = vibrance(out, 1.0 + tone * 0.12);
    out = contrast(out, 1.0 - tone * 0.04);
    return vec4<f32>(out, c.a);
}
