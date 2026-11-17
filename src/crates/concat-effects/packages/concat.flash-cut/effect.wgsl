struct Params { strength: f32 }

// A sharp white flash fires exactly at the cut, then the new shot lands.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.strength / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let d = (progress - 0.5) * 8.0;
    let flash = exp(-d * d);
    let base = mix(from_at(uv), to_at(uv), p);
    return vec4<f32>(mix(base.rgb, vec3<f32>(1.0), flash * 0.9 * k), base.a);
}
