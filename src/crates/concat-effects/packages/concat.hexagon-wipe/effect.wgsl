struct Params { soft: f32 }

// A hexagon edge grows from the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let q = abs(uv.x - 0.5);
    let r = abs(uv.y - 0.5);
    let d = max(q, r * 0.866 + q * 0.5);
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p * 0.9);
    return mix(from_at(uv), to_at(uv), m);
}
