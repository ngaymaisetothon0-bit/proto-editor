struct Params { amount: f32 }

// The picture tears into slices and loses signal, then the new shot locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let slice = floor(uv.y * 24.0);
    let n = hash(vec2<f32>(slice, floor(progress * 10.0)), 3.0);
    let tear = (n - 0.5) * 0.15 * sin(p * 3.14159) * k;
    let jump = (step(0.6, n) - 0.5) * sin(p * 3.14159) * k;
    let m = clamp(p + jump, 0.0, 1.0);
    let from_c = from_at(vec2<f32>(uv.x + tear, uv.y));
    return mix(from_c, to_at(uv), m);
}
