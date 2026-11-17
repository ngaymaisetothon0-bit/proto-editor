struct Params { tone: f32, glass: f32 }

// Obsidian, black volcanic glass: black and white, sharp and dark, like a mirror of stone.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.32, 0.36));
    var out = fade(g, params.glass / 100.0 * 0.03);
    out = contrast(out, 1.0 + tone * 0.11);
    return vec4<f32>(out, c.a);
}
