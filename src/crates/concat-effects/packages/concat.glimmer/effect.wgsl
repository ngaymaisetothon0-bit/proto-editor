struct Params { tone: f32, shine: f32 }

// Light catching on glass: bright, cool, sparkling at the edges, softly lifted.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.07);
    out = contrast(out, 1.0 - tone * 0.08);
    out = saturation(out, 1.0 - tone * 0.05);
    out = white_balance(out, 6500.0 + params.shine * 12.0);
    return vec4<f32>(out, c.a);
}
