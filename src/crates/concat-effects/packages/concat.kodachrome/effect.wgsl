struct Params { tone: f32, contrast: f32 }

// Slide film: warm reds, deep yellows, a toe that never quite clips.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = white_balance(c.rgb, 6500.0 - tone * 1200.0);
    out = saturation(out, 1.0 + tone * 0.35);
    out = film_curve(out, tone * 0.35, tone * 0.25);
    out = s_curve(out, params.contrast / 100.0 * 0.5);
    out = vignette(out, uv, tone * 0.2);
    return vec4<f32>(out, c.a);
}
