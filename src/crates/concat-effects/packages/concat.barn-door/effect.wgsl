// Two doors slide apart from the centre, left and right, to reveal the
// incoming shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let edge = 0.5 * (1.0 - progress);
    let m = smoothstep(edge - 0.01, edge + 0.01, abs(uv.x - 0.5));
    return mix(from_at(uv), to_at(uv), m);
}
