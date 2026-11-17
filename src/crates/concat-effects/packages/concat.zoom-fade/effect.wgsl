struct Params { amount: f32 }

// The camera zooms into the new shot, fading through the midpoint.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let z = 1.0 + params.amount / 100.0 * 1.0 * progress;
    let coord = (uv - vec2<f32>(0.5)) / z + vec2<f32>(0.5);
    let fade = 1.0 - 0.8 * sin(progress * 3.14159);
    let col = mix(from_at(coord), to_at(coord), step(0.5, progress));
    return vec4<f32>(col.rgb * fade, col.a);
}
