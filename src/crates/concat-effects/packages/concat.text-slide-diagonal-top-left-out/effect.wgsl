struct Params { duration: f32 }

// The whole title slides away to the top left and fades out over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, t);
    let shift = (1.0 - p) * 0.2;
    let coord = uv + vec2<f32>(shift, shift);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * p, c.a * p);
}
