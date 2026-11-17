struct Params { tone: f32, crystal: f32 }

// Quartz crystal: black and white, clear and hard, light through stone.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.32, 0.37));
    var out = fade(g, params.crystal / 100.0 * 0.03);
    out = contrast(out, 1.0 + tone * 0.11);
    return vec4<f32>(out, c.a);
}
