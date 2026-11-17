struct Params { tone: f32, haze: f32 }

// Smoke curling through grey air: black and white, hazy and soft, like an old film.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.33, 0.36));
    var out = fade(g, params.haze / 100.0 * 0.08);
    out = contrast(out, 1.0 + tone * 0.08);
    return vec4<f32>(out, c.a);
}
