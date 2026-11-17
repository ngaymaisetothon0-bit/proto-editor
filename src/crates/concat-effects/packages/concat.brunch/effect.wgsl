struct Params { tone: f32, sunny: f32 }

// A lazy brunch morning: bright, warm, cheerful — sunlight on the table and nothing to rush for.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.05);
    out = white_balance(out, 6500.0 - params.sunny * 15.0);
    out = vibrance(out, 1.0 + tone * 0.18);
    out = contrast(out, 1.0 + tone * 0.04);
    return vec4<f32>(out, c.a);
}
