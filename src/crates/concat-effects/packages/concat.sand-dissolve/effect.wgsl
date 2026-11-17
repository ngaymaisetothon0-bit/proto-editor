struct Params { size: f32 }

// The old shot blows away grain by grain, like sand in the wind.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = 10.0 + params.size / 100.0 * 12.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 7.0);
    let m = smoothstep(n, n + 0.2, p);
    let drift = (1.0 - m) * 0.05;
    let from_c = from_at(vec2<f32>(clamp(uv.x + drift, 0.0, 1.0), uv.y));
    return mix(from_c, to_at(uv), m);
}
