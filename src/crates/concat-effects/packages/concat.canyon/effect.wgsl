struct Params { tone: f32, rock: f32 }

// Red rock canyon at noon: warm stone, deep shadows, the desert glowing.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.04, 0.0, -0.02), tone);
    out = saturation(out, 1.0 + tone * 0.15);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
