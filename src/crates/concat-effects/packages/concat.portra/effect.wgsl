struct Params { tone: f32, warmth: f32 }

// Portra 400: warm, muted, soft ends - the film look for people.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = white_balance(c.rgb, 6500.0 - params.warmth * 25.0);
    out = saturation(out, 1.0 - tone * 0.15);
    out = matte(out, tone * 0.05, 1.0);
    out = film_curve(out, tone * 0.15, tone * 0.2);
    out = contrast(out, 1.0 - tone * 0.1);
    return vec4<f32>(out, c.a);
}
