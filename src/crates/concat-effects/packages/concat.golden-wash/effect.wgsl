struct Params { glow: f32 }

// A warm band of evening light sweeps across the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let wash = vec3<f32>(1.0, 0.75, 0.4) * exp(-abs(uv.x - progress) * 6.0) * k * 0.25;
    return base + vec4<f32>(wash, 0.0);
}
