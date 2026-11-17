struct Params { tone: f32, fresh: f32 }

// A fresh morning in the meadow: soft greens, bright light, easy on the eyes.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.02, 0.03, -0.02), tone);
    out = vibrance(out, 1.0 + tone * 0.2);
    out = white_balance(out, 6500.0 + params.fresh * 8.0);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
