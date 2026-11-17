struct Params { soft: f32 }

// A star edge expands from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let dx = abs(uv.x - 0.5);
    let dy = abs(uv.y - 0.5);
    let d = max(dx + dy * 0.5, dx * 0.5 + dy);
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
