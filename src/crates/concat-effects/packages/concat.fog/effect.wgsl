struct Params { tone: f32, mist: f32 }

// Misty grey: low contrast, lifted blacks, the world seen through fog.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let g = mono(c.rgb, vec3<f32>(0.3, 0.33, 0.37));
    var out = matte(g, params.mist / 100.0 * 0.18, 1.0 - tone * 0.15);
    out = contrast(out, 1.0 - tone * 0.25);
    return vec4<f32>(out, c.a);
}
