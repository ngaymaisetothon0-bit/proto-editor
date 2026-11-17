struct Params { duration: f32 }

// The title appears one word at a time, in reading order, over the first second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    if (reveal_order(uv) > p) {
        return vec4<f32>(0.0);
    }
    return sample(uv);
}
