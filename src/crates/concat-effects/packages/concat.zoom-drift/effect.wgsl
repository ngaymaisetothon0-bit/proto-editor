struct Params { amount: f32 }

// The camera zooms and drifts into the new shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let a = params.amount / 100.0 * 0.5 * progress;
    let z = 1.0 + params.amount / 100.0 * 0.3 * progress;
    let c = cos(a);
    let s = sin(a);
    let off = uv - vec2<f32>(0.5);
    let rot = vec2<f32>(off.x * c - off.y * s, off.x * s + off.y * c);
    let coord = rot / z + vec2<f32>(0.5 + 0.1 * progress, 0.5);
    let col = mix(from_at(coord), to_at(coord), progress);
    return col;
}
