struct Params { duration: f32 }

// Each word softens into a blur and fades out, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    let r = texel() * (1.0 - v) * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(uv + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    return vec4<f32>(c.rgb * v, c.a * v);
}
