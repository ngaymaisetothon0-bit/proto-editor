struct Params { glow: f32 }

// A cool band of moonlight sweeps across the cut, calm and silver-blue.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let wash = vec3<f32>(0.4, 0.6, 1.0) * exp(-abs(uv.x - progress) * 6.0) * k * 0.25;
    return base + vec4<f32>(wash, 0.0);
}
