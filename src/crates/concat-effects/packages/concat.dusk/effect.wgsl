struct Params { tone: f32, blue: f32 }

// Blue hour after sunset: deep blue shadows, warm embers in the light, quiet and cinematic.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(-0.02, 0.0, 0.04), vec3<f32>(0.02, -0.01, -0.02), tone);
    out = saturation(out, 1.0 - tone * 0.08);
    out = contrast(out, 1.0 + tone * 0.08);
    out = matte(out, tone * 0.03, 1.0);
    return vec4<f32>(out, c.a);
}
