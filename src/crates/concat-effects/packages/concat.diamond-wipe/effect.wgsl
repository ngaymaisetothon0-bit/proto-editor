struct Params { softness: f32 }

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    // A diamond in Manhattan distance: 0 at the centre, 1 at the middle of
    // each edge, 2 at the corners, so it opens all the way out.
    let d = (abs(uv.x - 0.5) + abs(uv.y - 0.5)) * 2.0;
    let radius = p * 2.2;
    let soft = max(params.softness * 0.01, 0.02);
    let m = smoothstep(radius - soft, radius + soft, d);
    return mix(to_at(uv), from_at(uv), m);
}
