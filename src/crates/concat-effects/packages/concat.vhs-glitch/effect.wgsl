struct Params { amount: f32 }

// VHS tracking trouble at the cut: banded jitter, split channels, a white tracking line.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let band = floor(uv.y * 18.0);
    let jitter = (hash(vec2<f32>(band, floor(progress * 30.0)), 3.0) - 0.5) * k * 0.08;
    let ux = clamp(uv.x + jitter, 0.0, 1.0);
    let from_c = vec4<f32>(
        from_at(vec2<f32>(clamp(ux + k * 0.008, 0.0, 1.0), uv.y)).r,
        from_at(vec2<f32>(ux, uv.y)).g,
        from_at(vec2<f32>(clamp(ux - k * 0.008, 0.0, 1.0), uv.y)).b,
        from_at(vec2<f32>(ux, uv.y)).a);
    let to_c = to_at(vec2<f32>(clamp(uv.x - jitter, 0.0, 1.0), uv.y));
    let tracking = step(0.92, hash(vec2<f32>(band, 0.0), 11.0)) * k * 0.15;
    let base = mix(from_c, to_c, p);
    return base + vec4<f32>(vec3<f32>(tracking), 0.0);
}
