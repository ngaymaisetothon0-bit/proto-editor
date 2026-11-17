struct Params { tone: f32, shift: f32 }

// Light through a prism: hues gently turned, colour richer, the picture bending toward the rainbow.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = hue_rotate(c.rgb, params.shift * 0.3);
    out = saturation(out, 1.0 + tone * 0.25);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
