struct Params { duration: f32 }

// The whole title wipes away from top to bottom over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // The edge sweeps past the far side (1.02 at t = 1) so that at rest
    // the whole title is visible, soft band and all.
    let edge = t * 1.04 - 0.02;
    let vis = 1.0 - smoothstep(edge - 0.02, edge, uv.y);
    let c = sample(uv);
    return vec4<f32>(c.rgb * vis, c.a * vis);
}
