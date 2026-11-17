struct Params { strength: f32 }

// Both shots rush through the lens at once: the outgoing zooms in as the incoming zooms out.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.strength / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let center = vec2<f32>(0.5, 0.5);
    let from_uv = center + (uv - center) / (1.0 + p * 0.5 * k);
    let to_uv = center + (uv - center) / (1.0 + (1.0 - p) * 0.5 * k);
    return mix(from_at(from_uv), to_at(to_uv), p);
}
