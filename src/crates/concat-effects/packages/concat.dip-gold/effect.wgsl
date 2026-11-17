struct Params { warmth: f32 }

// The cut dips through warm gold, like a flash of evening light.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.warmth / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let dip = sin(p * 3.14159);
    let gold = vec3<f32>(1.0, 0.85, 0.6);
    let base = mix(from_at(uv), to_at(uv), p);
    return vec4<f32>(mix(base.rgb, gold, dip * 0.8 * k), base.a);
}
