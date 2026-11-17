struct Params { tone: f32, dawn: f32 }

// Early morning light: bright, fresh, a little golden — a new day starting.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = fade(c.rgb, tone * 0.05);
    out = white_balance(out, 6500.0 - params.dawn * 10.0);
    out = vibrance(out, 1.0 + tone * 0.14);
    out = contrast(out, 1.0 + tone * 0.05);
    return vec4<f32>(out, c.a);
}
