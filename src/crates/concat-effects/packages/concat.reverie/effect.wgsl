struct Params { tone: f32, dream: f32 }

// A daydream on film: soft, lavender, slightly unreal — a memory half-remembered.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.06);
    out = split_tone(out, vec3<f32>(0.01, -0.01, 0.02), vec3<f32>(0.02, 0.0, 0.01), tone);
    out = saturation(out, 1.0 - tone * 0.1);
    out = contrast(out, 1.0 - tone * 0.06);
    return vec4<f32>(out, c.a);
}
