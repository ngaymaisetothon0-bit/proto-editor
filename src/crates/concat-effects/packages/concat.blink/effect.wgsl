struct Params { dark: f32 }

// A quick blink to black at the cut, like a camera shutter closing and opening.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.dark / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let blink = step(0.45, progress) * step(progress, 0.55);
    let base = mix(from_at(uv), to_at(uv), p);
    return vec4<f32>(mix(base.rgb, vec3<f32>(0.0), blink * k), base.a);
}
