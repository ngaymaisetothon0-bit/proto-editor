struct Params { duration: f32 }

// The title untypes itself one word at a time, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    // The * 8.0 step matches Typewriter's snappy per-word step.
    let rr = 1.0 - reveal_order(uv);
    let a = 1.0 - clamp((e - rr * 0.75) * 8.0, 0.0, 1.0);
    let c = sample(uv);
    return vec4<f32>(c.rgb * a, c.a * a);
}
