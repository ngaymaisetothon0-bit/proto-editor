struct Params { distance: f32 }

// The old shot slides off to the left, the new one underneath.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dist = params.distance / 100.0;
    let coord = uv - vec2<f32>(dist * p, 0.0);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return to_at(uv);
    }
    return from_at(coord);
}
