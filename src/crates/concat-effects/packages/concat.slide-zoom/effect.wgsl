fn slide_zoom_uv(uv: vec2<f32>, scale: f32, shift: f32) -> vec2<f32> {
    return (uv - vec2<f32>(0.5)) / max(scale, 0.0001) + vec2<f32>(0.5) + vec2<f32>(shift, 0.0);
}

// The outgoing picture slides away and shrinks while the incoming one
// slides in and grows to full size.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let a = from_at(slide_zoom_uv(uv, 1.0 - p * 0.35, p * 0.9));
    let b = to_at(slide_zoom_uv(uv, 0.65 + p * 0.35, -(1.0 - p) * 0.9));
    return mix(a, b, p);
}
