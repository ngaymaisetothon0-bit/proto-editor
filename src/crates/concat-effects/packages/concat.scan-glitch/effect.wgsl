struct Params { amount: f32 }

// A bright scan band sweeps down as the channels split apart at the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let shift = k * 0.01;
    let from_c = vec4<f32>(
        from_at(vec2<f32>(clamp(uv.x + shift, 0.0, 1.0), uv.y)).r,
        from_at(uv).g,
        from_at(vec2<f32>(clamp(uv.x - shift, 0.0, 1.0), uv.y)).b,
        from_at(uv).a);
    let band = exp(-abs(uv.y - progress) * 20.0) * k * 0.3;
    let base = mix(from_c, to_at(uv), p);
    return base + vec4<f32>(vec3<f32>(band), 0.0);
}
