struct Params { glow: f32 }

// Twinkling stars flare across the cut like a night sky.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let star = step(0.6, hash(floor(uv * vec2<f32>(24.0, 18.0)), floor(progress * 6.0)));
    let twinkle = star * sin(p * 3.14159) * k * 0.5;
    return base + vec4<f32>(vec3<f32>(1.0) * twinkle, 0.0);
}
