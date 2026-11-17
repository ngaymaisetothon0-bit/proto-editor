struct Params { duration: f32 }

// Standard easeOutBounce: a sequence of parabolas peaking at 0.75,
// 0.9375, 0.984375 and 1.0, so it stays in [0, 1] and reaches exactly
// 1.0 at x = 1 - the title settles at its rest position, never past it.
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
// The whole title rises in from below and bounces into place over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let shift = (1.0 - ease_out_bounce(p)) * 0.2;
    // The from-above bounce (text-bounce-in) uses uv + (0, shift);
    // from below flips the sign.
    let coord = uv - vec2<f32>(0.0, shift);
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * p, c.a * p);
}
