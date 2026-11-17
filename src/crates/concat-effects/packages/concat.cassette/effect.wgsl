struct Params { tone: f32, retro: f32 }

// An 80s cassette tape: warm, slightly washed out, a little hiss in the colours.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.03, 0.01, -0.02), tone);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = saturation(out, 1.0 - tone * 0.12);
    out = matte(out, tone * 0.04, 1.0 - tone * 0.03);
    return vec4<f32>(out, c.a);
}
