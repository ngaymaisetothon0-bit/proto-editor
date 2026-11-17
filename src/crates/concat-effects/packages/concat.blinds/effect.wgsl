// Horizontal slats open one after another, top to bottom, each sweeping
// down its own band to reveal the incoming shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let bands = 8.0;
    let band = floor(uv.y * bands);
    let p = clamp(progress * bands - band, 0.0, 1.0);
    let local = fract(uv.y * bands);
    let m = smoothstep(local - 0.03, local + 0.03, p);
    return mix(from_at(uv), to_at(uv), m);
}
