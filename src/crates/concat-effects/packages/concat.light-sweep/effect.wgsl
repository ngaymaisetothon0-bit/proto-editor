// A bright diagonal band sweeps across as the shots cross-fade, like a
// light passing over the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let d = (uv.x + uv.y) * 0.5;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let glow = exp(-abs(d - p) * 25.0) * 0.5;
    return base + vec4<f32>(vec3<f32>(glow), 0.0);
}
