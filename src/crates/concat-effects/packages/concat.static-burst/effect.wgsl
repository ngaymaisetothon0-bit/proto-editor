struct Params { amount: f32 }

// A burst of TV static crackles across the cut, then the new shot locks in.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.amount / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let base = mix(from_at(uv), to_at(uv), p);
    let noise = hash(floor(uv * vec2<f32>(64.0, 48.0)), floor(progress * 24.0));
    let burst = exp(-abs(progress - 0.5) * 12.0) * k * 0.5;
    return vec4<f32>(mix(base.rgb, vec3<f32>(noise), burst), base.a);
}
