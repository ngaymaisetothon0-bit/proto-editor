struct Params { size: f32 }

// The next shot shatters in as crystal shards, a few glints catching the light.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let cells = 8.0 + params.size / 100.0 * 8.0;
    let n = hash(floor(uv * vec2<f32>(cells, cells * 0.75)), 6.0);
    let m = smoothstep(n, n + 0.15, p);
    let sparkle = step(0.95, hash(floor(uv * vec2<f32>(24.0, 18.0)), 3.0)) * sin(p * 3.14159) * 0.3;
    let base = mix(from_at(uv), to_at(uv), m);
    return base + vec4<f32>(vec3<f32>(sparkle), 0.0);
}
