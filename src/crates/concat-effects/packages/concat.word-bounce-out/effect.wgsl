struct Params { duration: f32 }

// Standard easeOutBounce: a sequence of parabolas peaking at 0.75,
// 0.9375, 0.984375 and 1.0, so it stays in [0, 1] and reaches exactly
// 1.0 at x = 1 - the word settles at its rest position, never past it.
fn ease_out_bounce(x: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if (x < 1.0 / d1) {
        return n1 * x * x;
    } else if (x < 2.0 / d1) {
        return n1 * (x - 1.5 / d1) * (x - 1.5 / d1) + 0.75;
    } else if (x < 2.5 / d1) {
        return n1 * (x - 2.25 / d1) * (x - 2.25 / d1) + 0.9375;
    } else {
        return n1 * (x - 2.625 / d1) * (x - 2.625 / d1) + 0.984375;
    }
}
// Each word bounces upward and away, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    let shift = ease_out_bounce(1.0 - v) * 0.2;
    let coord = uv + vec2<f32>(0.0, shift);
    // The sampler clamps to the edge, so a coord outside the picture
    // would smear its edge instead of showing nothing: say so plainly.
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * v, c.a * v);
}
