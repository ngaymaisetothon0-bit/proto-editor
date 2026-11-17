struct Params { depth: f32 }

// The shots slide past each other at different speeds, like layers at different depths.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.depth / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let from_uv = vec2<f32>(clamp(uv.x + p * (0.6 + k * 0.4), 0.0, 1.0), uv.y);
    let to_uv = vec2<f32>(clamp(uv.x - (1.0 - p) * (0.4 + k * 0.4), 0.0, 1.0), uv.y);
    return mix(from_at(from_uv), to_at(to_uv), p);
}
