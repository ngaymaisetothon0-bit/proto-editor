struct Params { duration: f32 }

// The whole title wipes into view from the top-left corner over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    // The edge sweeps past the far corner (2.04 at p = 1, while
    // uv.x + uv.y tops out at 2.0) so that at the end the whole title
    // is visible, soft band and all.
    let edge = p * 2.08 - 0.04;
    let vis = 1.0 - smoothstep(edge - 0.04, edge, uv.x + uv.y);
    let c = sample(uv);
    return vec4<f32>(c.rgb * vis, c.a * vis);
}
