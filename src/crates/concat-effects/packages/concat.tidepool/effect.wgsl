struct Params { tone: f32, pool: f32 }

// A tidepool at low tide: green-blue water, wet rock, life in miniature.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.01, 0.02, 0.01), tone);
    out = saturation(out, 1.0 + tone * 0.14);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
