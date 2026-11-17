struct Params { soft: f32 }

// A diagonal edge with a zigzag bite sweeps across the cut.
fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = smoothstep(0.0, 1.0, progress);
    let zig = sin(uv.y * 25.1327) * 0.03;
    let d = (uv.x + uv.y) * 0.5 + zig;
    let edge = 0.02 + params.soft / 100.0 * 0.08;
    let m = smoothstep(d - edge, d + edge, p);
    return mix(from_at(uv), to_at(uv), m);
}
