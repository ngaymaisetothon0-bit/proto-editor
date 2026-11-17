struct Params { tone: f32, breeze: f32 }

// Sea air on the coast: cool blues and fresh greens lifted, crisp light like a walk by the water.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = white_balance(c.rgb, 6500.0 + params.breeze * 25.0);
    out = vibrance(out, 1.0 + tone * 0.22);
    out = s_curve(out, params.breeze / 100.0 * 0.3);
    out = saturation(out, 1.0 + tone * 0.08);
    return vec4<f32>(out, c.a);
}
