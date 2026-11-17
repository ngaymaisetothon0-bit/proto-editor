struct Params { tone: f32, threshold: f32 }

// The darkroom accident: tones above the threshold flip to their negative.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let tone = params.tone / 100.0;
    let th = params.threshold / 100.0;
    let l = luma(c.rgb);
    let flipped = mix(c.rgb, vec3<f32>(1.0) - c.rgb, step(th, l));
    var out = mix(c.rgb, flipped, tone);
    out = s_curve(out, 0.15);
    return vec4<f32>(out, c.a);
}
