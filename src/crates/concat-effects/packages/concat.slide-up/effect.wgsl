struct Params { distance: f32 }

// The new shot slides up from below, pushing the old one off the top.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dist = params.distance / 100.0;
    let d = vec2<f32>(0.0, dist * (1.0 - p));
    let coord = uv - d;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return from_at(uv);
    }
    return to_at(coord);
}
