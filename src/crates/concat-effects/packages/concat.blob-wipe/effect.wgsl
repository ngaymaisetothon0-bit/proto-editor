struct Params { soft: f32 }

// A blob edge expands from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let d = length((uv - vec2<f32>(0.5)) * vec2<f32>(1.0, 1.2)) + 0.05 * sin(atan2(uv.y - 0.5, uv.x - 0.5) * 3.0);
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
