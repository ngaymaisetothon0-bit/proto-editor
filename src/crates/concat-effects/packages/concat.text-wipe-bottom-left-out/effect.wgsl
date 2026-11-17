struct Params { duration: f32 }

// The whole title wipes away from the bottom-left corner over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // The edge sweeps from the far corner (-1.04 at t = 0) up past
    // the near corner (1.04 at t = 1) so that at rest the whole title
    // is visible, soft band and all.
    let edge = -1.04 + t * 2.08;
    let vis = 1.0 - smoothstep(edge - 0.04, edge, uv.x - uv.y);
    let c = sample(uv);
    return vec4<f32>(c.rgb * vis, c.a * vis);
}
