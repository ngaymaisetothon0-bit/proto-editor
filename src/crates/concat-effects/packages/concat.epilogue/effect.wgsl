struct Params { tone: f32, close: f32 }

// The epilogue of a long film: warm shadows, cool light, the story closing gently.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.02, 0.0, -0.01), vec3<f32>(-0.01, 0.0, 0.02), tone);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = saturation(out, 1.0 - tone * 0.12);
    out = vignette(out, uv, tone * 0.35);
    return vec4<f32>(out, c.a);
}
