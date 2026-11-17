struct Params { tone: f32, dust: f32 }

// Dusty western sun: warm browns, faded film, high noon on the frontier.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.01, -0.04), tone);
    out = matte(out, tone * 0.06, 1.0 - tone * 0.05);
    out = film_curve(out, tone * 0.3, tone * 0.2);
    out = saturation(out, 1.0 - tone * 0.1);
    return vec4<f32>(out, c.a);
}
