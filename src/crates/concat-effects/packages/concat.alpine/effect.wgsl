struct Params { tone: f32, crisp: f32 }

// Mountain air: cool, crisp, the blues and greens lifted.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = white_balance(c.rgb, 6500.0 + params.crisp * 30.0);
    out = vibrance(out, 1.0 + tone * 0.25);
    out = s_curve(out, params.crisp / 100.0 * 0.4);
    out = contrast(out, 1.0 + tone * 0.1);
    return vec4<f32>(out, c.a);
}
