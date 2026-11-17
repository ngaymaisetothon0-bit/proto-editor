struct Params { size: f32 }

// The next shot is painted in, block by block, in a scattered brush-stroke order.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let cells = 8.0 + params.size / 100.0 * 24.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 7.0);
    let m = smoothstep(n, n + 0.25, smoothstep(0.0, 1.0, progress));
    return mix(from_at(uv), to_at(uv), m);
}
