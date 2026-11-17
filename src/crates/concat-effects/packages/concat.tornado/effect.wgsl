struct Params { twist: f32 }

// The frame twists into a spiral vortex at the middle of the cut, then unwinds.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let k = sin(progress * 3.14159) * params.twist / 100.0;
    let p = smoothstep(0.0, 1.0, progress);
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    let ang = (1.0 - d) * k * 2.0;
    let c = cos(ang);
    let s = sin(ang);
    let off = uv - vec2<f32>(0.5, 0.5);
    let rot = vec2<f32>(off.x * c - off.y * s, off.x * s + off.y * c) + vec2<f32>(0.5, 0.5);
    let ru = clamp(rot, vec2<f32>(0.0), vec2<f32>(1.0));
    return mix(from_at(ru), to_at(ru), p);
}
