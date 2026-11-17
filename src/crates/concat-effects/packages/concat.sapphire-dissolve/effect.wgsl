struct Params { size: f32 }

// Sapphire flakes wash across the frame, the new shot underneath.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = 8.0 + params.size / 100.0 * 4.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 107.0);
    let m = smoothstep(n, min(n + 0.2, 1.0), p);
    let band = clamp(1.0 - abs(p - n) * 10.0, 0.0, 1.0) * (1.0 - p);
    let col = mix(from_at(uv), to_at(uv), m);
    return vec4<f32>(mix(col.rgb, vec3<f32>(0.2, 0.35, 0.85), band * 0.35), col.a);
}
