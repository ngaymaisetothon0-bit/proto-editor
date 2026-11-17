struct Params { tone: f32, nostalgia: f32 }

// A drive-in movie night: warm, a little faded, pure nostalgia.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(0.05, 0.02, -0.03), tone);
    out = matte(out, tone * 0.05, 1.0 - tone * 0.04);
    out = film_curve(out, tone * 0.25, tone * 0.2);
    out = saturation(out, 1.0 - tone * 0.08);
    return vec4<f32>(out, c.a);
}
