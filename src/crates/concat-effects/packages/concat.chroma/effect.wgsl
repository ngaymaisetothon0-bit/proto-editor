struct Params { tone: f32, punch: f32 }

// Colour turned all the way up: rich, saturated, every hue singing a little louder.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = saturation(c.rgb, 1.0 + tone * 0.35);
    out = vibrance(out, 1.0 + tone * 0.15);
    out = contrast(out, 1.0 + tone * 0.06);
    return vec4<f32>(out, c.a);
}
