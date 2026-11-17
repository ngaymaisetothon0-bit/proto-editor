struct Params { tone: f32, ember: f32 }

// The glow after sunset: warm embers below, cool dusk above — the day's last light.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.02, 0.0, -0.02), vec3<f32>(-0.01, 0.0, 0.02), tone);
    out = fade(out, tone * 0.05);
    out = saturation(out, 1.0 - tone * 0.08);
    out = contrast(out, 1.0 + tone * 0.06);
    return vec4<f32>(out, c.a);
}
