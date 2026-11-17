struct Params { size: f32 }

// The old shot frosts over and melts away flake by flake.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = 8.0 + params.size / 100.0 * 8.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 7.0);
    let m = smoothstep(n, min(n + 0.15, 1.0), p);
    let frost = clamp(1.0 - abs(p - n) * 10.0, 0.0, 1.0) * (1.0 - p);
    let col = mix(from_at(uv), to_at(uv), m);
    return vec4<f32>(mix(col.rgb, vec3<f32>(0.95, 0.97, 1.0), frost * 0.35), col.a);
}
