struct Params { amount: f32 }

// The camera zooms and orbits into the new shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let a = params.amount / 100.0 * 0.5 * progress;
    let z = 1.0 + params.amount / 100.0 * 0.3 * sin(progress * 3.14159);
    let c = cos(a);
    let s = sin(a);
    let off = uv - vec2<f32>(0.5);
    let rot = vec2<f32>(off.x * c - off.y * s, off.x * s + off.y * c);
    let coord = rot / z + vec2<f32>(0.5 + 0.08 * sin(progress * 6.28318), 0.5);
    let col = mix(from_at(coord), to_at(coord), progress);
    return col;
}
