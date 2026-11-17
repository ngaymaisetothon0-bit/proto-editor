struct Params { tone: f32, lush: f32 }

// Lush green: forests and fields go deep green, the rest stays honest.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = tint_midtones(c.rgb, vec3<f32>(-0.03, 0.04, -0.01), tone);
    out = vibrance(out, 1.0 + tone * 0.3);
    out = white_balance(out, 6500.0 + params.lush * 15.0);
    out = s_curve(out, tone * 0.3);
    return vec4<f32>(out, c.a);
}
