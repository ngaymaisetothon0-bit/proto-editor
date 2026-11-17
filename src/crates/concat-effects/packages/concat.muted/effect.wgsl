struct Params { tone: f32, soft: f32 }

// Quiet and editorial: soft desaturated tones, gentle contrast, nothing shouting.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 - tone * 0.3);
    out = matte(out, tone * 0.05, 1.0 - tone * 0.03);
    out = contrast(out, 1.0 - tone * 0.05);
    return vec4<f32>(out, c.a);
}
