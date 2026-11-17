// The picture breaks into blocks that flip to the incoming shot one
// after another, scattered so no straight edge sweeps across.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let cells = 16.0;
    let cell = (floor(uv * cells) + 0.5) / cells;
    let order = fract((floor(uv.x * cells) + floor(uv.y * cells)) * 0.377);
    let m = smoothstep(order, order + 0.15, progress);
    return mix(from_at(cell), to_at(cell), m);
}
