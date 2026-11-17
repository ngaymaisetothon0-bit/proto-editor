struct Params { tone: f32, paper: f32 }

// Paper cutout: flat, bold, graphic — colour cut with scissors and stuck on the page.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.25);
    out = contrast(out, 1.0 + tone * 0.12);
    out = fade(out, tone * 0.03);
    out = vignette(out, uv, tone * 0.25);
    return vec4<f32>(out, c.a);
}
