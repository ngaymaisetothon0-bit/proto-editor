struct Params { tone: f32, beam: f32 }

// A shaft of sunlight: warm, bright, lifted like morning through a window.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.06);
    out = white_balance(out, 6500.0 - params.beam * 15.0);
    out = vibrance(out, 1.0 + tone * 0.12);
    out = contrast(out, 1.0 + tone * 0.04);
    return vec4<f32>(out, c.a);
}
