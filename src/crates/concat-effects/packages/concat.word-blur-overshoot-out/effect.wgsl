struct Params { duration: f32 }

// Standard easeOutBack: grows past 1.0 once (peaks at ~1.1001) and
// settles back to exactly 1.0 at x = 1 - the title overshoots full
// size once, then rests.
fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let u = x - 1.0;
    return 1.0 + c3 * u * u * u + c1 * u * u;
}
// Each word overshoots out to 60%, past it once, while blurring away, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    // At rest (v = 1) the overshoot term is 0, so the scale is exactly
    // 1.0 and the word sits at its rest size.
    let scale = 1.0 - 0.4 * ease_out_back(1.0 - v);
    let coord = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) / scale;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    // The same 3x3 box blur text-blur-in uses: at full progress the
    // radius is 0, so all nine taps read the same pixel and the
    // picture is exactly sharp.
    let r = texel() * (1.0 - v) * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(coord + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    return vec4<f32>(c.rgb * v, c.a * v);
}
