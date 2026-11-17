struct Params { tone: f32, age: f32 }

// The silent era: warm sepia grey, soft ends, a little age in the blacks.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.32, 0.34, 0.34));
    var out = tint_midtones(g, vec3<f32>(0.05, 0.03, -0.02), tone);
    out = matte(out, params.age / 100.0 * 0.1, 1.0 - tone * 0.05);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = contrast(out, 1.0 - tone * 0.08);
    return vec4<f32>(out, c.a);
}
