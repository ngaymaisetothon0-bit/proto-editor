struct Params { distance: f32 }

// The shots slide past each other on the diagonal, one out top-right, one in bottom-left.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.distance / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let from_uv = vec2<f32>(clamp(uv.x + p * (0.3 + k * 0.4), 0.0, 1.0), clamp(uv.y + p * (0.3 + k * 0.4), 0.0, 1.0));
    let to_uv = vec2<f32>(clamp(uv.x - (1.0 - p) * (0.3 + k * 0.4), 0.0, 1.0), clamp(uv.y - (1.0 - p) * (0.3 + k * 0.4), 0.0, 1.0));
    return mix(from_at(from_uv), to_at(to_uv), p);
}
