struct Params { tone: f32, night: f32 }

// A nocturne after dark: deep blue shadows, quiet streets, the city asleep.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(-0.02, -0.01, 0.03), vec3<f32>(0.0, 0.01, 0.02), tone);
    out = saturation(out, 1.0 - tone * 0.14);
    out = contrast(out, 1.0 + tone * 0.1);
    out = vignette(out, uv, tone * 0.4);
    return vec4<f32>(out, c.a);
}
