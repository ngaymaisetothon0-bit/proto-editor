struct Params { glow: f32 }

// A rainbow wash sweeps across the cut, bright and playful.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let t = uv.x * 6.28318 + progress * 6.28318;
    let col = vec3<f32>(
        0.5 + 0.5 * sin(t),
        0.5 + 0.5 * sin(t + 2.094),
        0.5 + 0.5 * sin(t + 4.189));
    let wash = col * k * 0.25;
    return base + vec4<f32>(wash, 0.0);
}
