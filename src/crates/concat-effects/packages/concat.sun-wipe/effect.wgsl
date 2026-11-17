struct Params { soft: f32 }

// A sun edge expands from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dx = uv.x - 0.5;
    let dy = uv.y - 0.5;
    let d = length(uv - vec2<f32>(0.5)) + 0.04 * sin(atan2(dy, dx) * 8.0);
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
