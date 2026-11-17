struct Params { duration: f32 }

// The clip slides up out of the frame over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // Cubic ease-out: fast off the mark, soft on landing.
    let p = 1.0 - pow(1.0 - t, 3.0);
    let shift = (1.0 - p) * 1.0;
    let coord = uv + vec2<f32>(0.0, shift);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    return sample(coord);
}
