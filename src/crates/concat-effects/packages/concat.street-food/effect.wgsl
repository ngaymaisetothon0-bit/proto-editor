struct Params { tone: f32, warmth: f32 }

// Street food at noon: warm, vivid, a little extra contrast so the food pops.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.35);
    out = white_balance(out, 6500.0 - params.warmth * 30.0);
    out = contrast(out, 1.0 + tone * 0.15);
    return vec4<f32>(out, c.a);
}
