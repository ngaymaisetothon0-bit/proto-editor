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
// The whole title overshoots out, shrinking past 60% once and away over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // At rest (t = 1) the overshoot term is 0, so the scale is exactly
    // 1.0 and the title sits at its rest size.
    let scale = 1.0 - 0.4 * ease_out_back(1.0 - t);
    let coord = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) / scale;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * t, c.a * t);
}
