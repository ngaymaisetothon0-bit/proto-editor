struct Params { amount: f32 }

// The camera crash-zooms into the new shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let z = 1.0 + params.amount / 100.0 * 1.5 * progress;
    let coord = (uv - vec2<f32>(0.5)) / z + vec2<f32>(0.5);
    let col = mix(from_at(coord), to_at(coord), step(0.5, progress));
    return col;
}
