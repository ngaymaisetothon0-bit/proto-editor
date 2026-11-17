struct Params { amount: f32 }

// The camera dives down a tunnel into the new shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let z = 1.0 + params.amount / 100.0 * 1.2 * progress;
    let coord = (uv - vec2<f32>(0.5)) / z + vec2<f32>(0.5);
    let dist = length(uv - vec2<f32>(0.5)) * 2.0;
    let vig = 1.0 - 0.3 * sin(progress * 3.14159) * smoothstep(0.5, 1.0, dist);
    let col = mix(from_at(coord), to_at(coord), step(0.5, progress));
    return vec4<f32>(col.rgb * vig, col.a);
}
