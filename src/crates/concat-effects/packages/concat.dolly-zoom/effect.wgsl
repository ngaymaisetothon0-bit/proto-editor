struct Params { amount: f32 }

// The frame zooms in a vertigo pulse as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let z = 1.0 + params.amount / 100.0 * 0.6 * sin(progress * 3.14159);
    let coord = (uv - vec2<f32>(0.5)) / z + vec2<f32>(0.5);
    let col = mix(from_at(coord), to_at(coord), step(0.5, progress));
    return col;
}
