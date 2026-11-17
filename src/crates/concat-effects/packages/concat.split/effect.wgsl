// The outgoing shot splits into top and bottom halves that slide apart,
// the incoming shot showing through the gap between them.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let gap = progress * 0.5;
    if (abs(uv.y - 0.5) < gap) {
        return to_at(uv);
    }
    let shift = vec2<f32>(0.0, gap);
    if (uv.y < 0.5) {
        return from_at(uv + shift);
    }
    return from_at(uv - shift);
}
