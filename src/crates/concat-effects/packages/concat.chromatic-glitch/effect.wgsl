struct Params { amount: f32 }

// The colour channels split and jump as the new shot glitches in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(vec2<f32>(0.0, floor(progress * 6.0)), 7.0);
    let off = (n - 0.5) * 0.05 * k;
    let jump = (n - 0.5) * 1.5 * sin(p * 3.14159) * k;
    let m = clamp(p + jump, 0.0, 1.0);
    let r = from_at(vec2<f32>(uv.x + off, uv.y)).r;
    let g = from_at(uv).g;
    let b = from_at(vec2<f32>(uv.x - off, uv.y)).b;
    let from_c = vec4<f32>(r, g, b, 1.0);
    return mix(from_c, to_at(uv), m);
}
