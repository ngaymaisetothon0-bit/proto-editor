struct Params { strength: f32 }

// Green and violet light washes over the cut like the northern lights.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.strength / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let band = sin((uv.y * 3.0 + progress * 2.0) * 3.14159) * 0.5 + 0.5;
    let green = vec3<f32>(0.1, 0.9, 0.4) * band;
    let purple = vec3<f32>(0.6, 0.2, 0.9) * (1.0 - band);
    let wash = (green + purple) * k * 0.25 * (1.0 - abs(uv.x - 0.5));
    return base + vec4<f32>(wash, 0.0);
}
