struct Params { tone: f32, contrast: f32 }

// Infrared false colour: green foliage turns red-orange, the sky stays blue.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let mapped = vec3<f32>(max(c.r, c.g), mix(c.g, c.b, 0.5), c.b);
    var out = mix(c.rgb, mapped, tone);
    out = saturation(out, 1.0 + tone * 0.25);
    out = s_curve(out, params.contrast / 100.0 * 0.6);
    return vec4<f32>(out, c.a);
}
