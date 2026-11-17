struct Params { amount: f32 }

// Worn tape tracking bands shift and glitch as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(vec2<f32>(0.0, floor(progress * 6.0)), 13.0);
    let shift = (hash(vec2<f32>(floor(uv.y * 16.0), floor(progress * 10.0)), 3.0) - 0.5) * 0.15 * k * sin(p * 3.14159);
    let jump = (n - 0.5) * 1.5 * sin(p * 3.14159) * k;
    let m = clamp(p + jump, 0.0, 1.0);
    return mix(from_at(vec2<f32>(uv.x + shift, uv.y)), to_at(uv), m);
}
