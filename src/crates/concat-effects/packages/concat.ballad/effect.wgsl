struct Params { tone: f32, song: f32 }

// A ballad on the big screen: warm light, deep shadows, a song told in pictures.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    var out = split_tone(c.rgb, vec3<f32>(0.02, 0.0, -0.01), vec3<f32>(-0.01, 0.0, 0.02), tone);
    out = film_curve(out, tone * 0.2, tone * 0.15);
    out = saturation(out, 1.0 - tone * 0.12);
    out = contrast(out, 1.0 + tone * 0.1);
    out = vignette(out, uv, tone * 0.35);
    return vec4<f32>(out, c.a);
}
