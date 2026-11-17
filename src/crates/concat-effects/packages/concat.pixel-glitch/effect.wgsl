struct Params { amount: f32 }

// The picture breaks into glitching blocks, then the new shot locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(vec2<f32>(0.0, floor(progress * 6.0)), 9.0);
    let q = 8.0 + (1.0 - sin(p * 3.14159)) * 24.0;
    let uvq = floor(uv * q) / q;
    let jump = (n - 0.5) * 1.5 * sin(p * 3.14159) * k;
    let m = clamp(p + jump, 0.0, 1.0);
    return mix(from_at(uvq), to_at(uv), m);
}
