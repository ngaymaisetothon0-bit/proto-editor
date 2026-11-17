struct Params { duration: f32 }

// The title leaves one word at a time, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    if (reveal_order(uv) > p) {
        return vec4<f32>(0.0);
    }
    return sample(uv);
}
