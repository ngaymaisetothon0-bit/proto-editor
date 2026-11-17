struct Params { softness: f32 }

// The next shot drifts in through smoke, soft large clouds instead of hard blocks.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(floor(uv * vec2<f32>(3.0, 2.0)), 2.0) * 0.7
        + hash(floor(uv * vec2<f32>(9.0, 6.0)), 7.0) * 0.3;
    let w = 0.15 + params.softness / 100.0 * 0.2;
    let m = smoothstep(n - w, n + w, p);
    let glow = exp(-abs(n - p) * 8.0) * 0.12;
    let base = mix(from_at(uv), to_at(uv), m);
    return base + vec4<f32>(vec3<f32>(glow), 0.0);
}
