struct Params { glow: f32 }

// A bright comet streaks diagonally across the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let streak = exp(-abs((uv.x + uv.y) * 0.5 - progress) * 8.0) * k * 0.35;
    return base + vec4<f32>(vec3<f32>(0.9, 0.95, 1.0) * streak, 0.0);
}
