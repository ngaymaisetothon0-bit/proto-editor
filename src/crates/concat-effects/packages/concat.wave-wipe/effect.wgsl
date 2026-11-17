struct Params { soft: f32 }

// A wave edge expands from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let d = max(abs(uv.x - 0.5), abs(uv.y - 0.5)) + 0.05 * sin(uv.x * 20.0 + p * 3.0);
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
