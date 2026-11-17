struct Params { size: f32 }

// The old shot melts away drip by drip, the new one underneath.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = 10.0 + params.size / 100.0 * 12.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 11.0);
    let m = smoothstep(n, n + 0.2, p);
    let drip = (1.0 - m) * 0.08;
    let from_c = from_at(vec2<f32>(uv.x, clamp(uv.y - drip, 0.0, 1.0)));
    return mix(from_c, to_at(uv), m);
}
