struct Params { duration: f32 }

// Each word shrinks away and fades out, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    // Full size while the word is there, 60% of size when it is gone.
    let scale = 1.0 - 0.4 * (1.0 - v);
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let coord = c / scale / frame.size + vec2<f32>(0.5);
    let out = sample(coord);
    return vec4<f32>(out.rgb * v, out.a * v);
}
