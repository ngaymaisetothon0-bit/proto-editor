struct Params { amount: f32 }

// Rolling interference bands chew the picture, then the new shot locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(vec2<f32>(0.0, floor(progress * 10.0)), 4.0);
    let wobble = sin((uv.y * 8.0 + progress * 6.28318) * 3.14159) * 0.02 * sin(p * 3.14159) * k;
    let jump = (step(0.5, n) - 0.5) * sin(p * 3.14159) * k;
    let m = clamp(p + jump, 0.0, 1.0);
    let from_c = from_at(vec2<f32>(uv.x + wobble, uv.y));
    return mix(from_c, to_at(uv), m);
}
