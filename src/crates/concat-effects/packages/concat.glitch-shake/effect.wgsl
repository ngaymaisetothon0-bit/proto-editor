struct Params { amount: f32 }

// The picture shakes apart and the cut stutters, then locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let n = hash(vec2<f32>(0.0, floor(progress * 12.0)), 9.0);
    let shake = (n - 0.5) * 0.06 * sin(progress * 25.1327) * k;
    let m = clamp(p + (n - 0.5) * 2.0 * sin(p * 3.14159) * k, 0.0, 1.0);
    let from_c = from_at(vec2<f32>(uv.x + shake, uv.y));
    return mix(from_c, to_at(uv), m);
}
