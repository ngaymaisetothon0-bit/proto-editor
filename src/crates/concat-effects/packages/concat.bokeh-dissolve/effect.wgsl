struct Params { amount: f32 }

// A dreamy dissolve: the picture blooms soft and bright at the middle of the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let t = texel() * (1.0 + k * 12.0);
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            let off = vec2<f32>(f32(x), f32(y)) * t;
            sum += mix(from_at(uv + off), to_at(uv + off), p);
        }
    }
    let blurred = sum / 9.0;
    let base = mix(from_at(uv), to_at(uv), p);
    let out = mix(base.rgb, blurred.rgb, k * 0.6);
    return vec4<f32>(out + vec3<f32>(k * 0.06), base.a);
}
