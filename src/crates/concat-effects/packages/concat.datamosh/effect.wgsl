struct Params { amount: f32 }

// Blocks of the picture glitch forward in time, then the new shot locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(floor(uv * vec2<f32>(16.0, 12.0)), floor(progress * 8.0));
    let mosh = step(0.55, n) * sin(p * 3.14159) * k;
    let m = clamp(p + mosh * 0.5, 0.0, 1.0);
    return mix(from_at(uv), to_at(uv), m);
}
