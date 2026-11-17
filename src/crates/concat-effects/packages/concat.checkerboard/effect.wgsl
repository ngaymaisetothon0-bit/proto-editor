// Squares flip to the incoming shot in diagonal order, from the top
// left corner to the bottom right.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let cells = 8.0;
    let ix = floor(uv.x * cells);
    let iy = floor(uv.y * cells);
    let order = (ix + iy) / (2.0 * cells - 2.0);
    let m = step(order, progress);
    return mix(from_at(uv), to_at(uv), m);
}
