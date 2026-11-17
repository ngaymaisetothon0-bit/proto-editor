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
// The whole title grows in from 60%, overshoots full size once, and settles over the first half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let scale = 0.6 + 0.4 * ease_out_back(p);
    let coord = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) / scale;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let c = sample(coord);
    return vec4<f32>(c.rgb * p, c.a * p);
}
