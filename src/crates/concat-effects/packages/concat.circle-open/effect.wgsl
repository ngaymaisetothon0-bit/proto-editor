// A circle opens from the middle of the frame to reveal the incoming shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let r = progress * 0.75;
    let m = 1.0 - smoothstep(r - 0.02, r + 0.02, d);
    return mix(from_at(uv), to_at(uv), m);
}
