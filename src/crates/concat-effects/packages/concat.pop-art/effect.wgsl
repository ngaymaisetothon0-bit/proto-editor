struct Params { tone: f32, punch: f32 }

// Bold pop art: punchy saturated colour, hard contrast, print-shop energy.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.5);
    out = contrast(out, 1.0 + tone * 0.2);
    out = s_curve(out, tone * 0.3);
    out = white_balance(out, 6500.0 - params.punch * 8.0);
    return vec4<f32>(out, c.a);
}
