struct Params { tone: f32, coin: f32 }

// Nickel, bright and grey: black and white, clean and hard, like a new coin.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.32, 0.37));
    var out = fade(g, params.coin / 100.0 * 0.03);
    out = contrast(out, 1.0 + tone * 0.1);
    return vec4<f32>(out, c.a);
}
