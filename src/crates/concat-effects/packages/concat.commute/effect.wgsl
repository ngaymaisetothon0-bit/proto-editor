struct Params { tone: f32, rush: f32 }

// The morning commute: grey streets, clear air, the city waking up.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.04);
    out = white_balance(out, 6500.0 - params.rush * 8.0);
    out = vibrance(out, 1.0 + tone * 0.12);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
