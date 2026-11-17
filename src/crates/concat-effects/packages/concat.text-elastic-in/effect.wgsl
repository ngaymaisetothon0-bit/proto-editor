struct Params { duration: f32 }

// Standard easeOutElastic: overshoots past 1.0 (peaks at ~1.3536),
// dips just below it (~0.875), and settles at exactly 1.0 - the
// title wobbles like a rubber band before it rests. The piecewise
// guards make the endpoints exactly 0 and 1 (the plain formula
// leaves ~1e-16 noise at x = 0).
fn ease_out_elastic(x: f32) -> f32 {
    if (x <= 0.0) {
        return 0.0;
    }
    if (x >= 1.0) {
        return 1.0;
    }
    let c4 = 6.2831853 / 3.0;
    return pow(2.0, -10.0 * x) * sin((x * 10.0 - 0.75) * c4) + 1.0;
}
// The whole title wobbles elastically into place, growing from 60% with a rubber-band overshoot, over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let scale = 0.6 + 0.4 * ease_out_elastic(p);
    let coord = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) / scale;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * p, c.a * p);
}
