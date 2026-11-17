struct Params { strength: f32 }

// The cut dips through pink, like a soft flash of colour.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.strength / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let dip = sin(p * 3.14159);
    let base = mix(from_at(uv), to_at(uv), p);
    return vec4<f32>(mix(base.rgb, vec3<f32>(1.0, 0.75, 0.8), dip * 0.9 * k), base.a);
}
