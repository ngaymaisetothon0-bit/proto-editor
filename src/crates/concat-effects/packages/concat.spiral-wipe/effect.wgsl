struct Params { soft: f32 }

// A spiral edge winds around the centre as the new shot takes over.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let ang = atan2(uv.y - 0.5, uv.x - 0.5) / 6.28318 + 0.5;
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(ang - edge, ang + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
