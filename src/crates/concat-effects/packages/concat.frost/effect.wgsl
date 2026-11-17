struct Params { tone: f32, contrast: f32 }

// A cold morning: paler, flatter, the air itself in the picture.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 - tone * 0.4);
    out = fade(out, tone * 0.06);
    out = split_tone(out, vec3<f32>(-0.02, 0.02, 0.07), vec3<f32>(0.02, 0.03, 0.02), tone);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
