struct Params { amount: f32 }

// The old shot sags like a stopping tape, channels splitting, then the cut snaps.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let sy = clamp(0.5 + (uv.y - 0.5) * (1.0 + k * 0.3), 0.0, 1.0);
    let shift = k * 0.01;
    let from_c = vec4<f32>(
        from_at(vec2<f32>(clamp(uv.x + shift, 0.0, 1.0), sy)).r,
        from_at(vec2<f32>(uv.x, sy)).g,
        from_at(vec2<f32>(clamp(uv.x - shift, 0.0, 1.0), sy)).b,
        from_at(vec2<f32>(uv.x, sy)).a);
    let base = mix(from_c, to_at(uv), p);
    return vec4<f32>(base.rgb * (1.0 + k * 0.15), base.a);
}
