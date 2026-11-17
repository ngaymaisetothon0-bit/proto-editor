struct Params { duration: f32 }

// The whole title fades out softly over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    let c = sample(uv);
    return vec4<f32>(c.rgb * p, c.a * p);
}
