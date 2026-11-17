struct Params { glow: f32 }

// A lens flare blooms at the middle of the cut as the shots cross-fade.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.glow / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let d = distance(uv, vec2<f32>(0.5, 0.42));
    let flare = exp(-d * 6.0) * k * 0.9;
    let streak = exp(-abs(uv.y - 0.42) * 30.0) * exp(-d * 2.0) * k * 0.35;
    return base + vec4<f32>(vec3<f32>(flare + streak), 0.0);
}
