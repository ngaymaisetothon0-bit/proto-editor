struct Params { spin: f32 }

// The frame spins like a pinwheel around the centre, then settles on the new shot.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = params.spin / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let ang = p * k * 1.5;
    let c = cos(ang);
    let s = sin(ang);
    let off = uv - vec2<f32>(0.5, 0.5);
    let rot = vec2<f32>(off.x * c - off.y * s, off.x * s + off.y * c) + vec2<f32>(0.5, 0.5);
    let ru = clamp(rot, vec2<f32>(0.0), vec2<f32>(1.0));
    return mix(from_at(ru), to_at(ru), p);
}
