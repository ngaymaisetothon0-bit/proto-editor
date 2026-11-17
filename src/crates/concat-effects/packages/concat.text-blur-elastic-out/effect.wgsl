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
// The whole title wobbles elastically away, shrinking to 60% with a rubber-band overshoot, while blurring away over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let t = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    // At rest (t = 1) the elastic term is 0, so the scale is exactly
    // 1.0 and the title sits at its rest size.
    let scale = 1.0 - 0.4 * ease_out_elastic(1.0 - t);
    let coord = vec2<f32>(0.5) + (uv - vec2<f32>(0.5)) / scale;
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    // The same 3x3 box blur text-blur-in uses: at full progress the
    // radius is 0, so all nine taps read the same pixel and the
    // picture is exactly sharp.
    let r = texel() * (1.0 - t) * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(coord + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    return vec4<f32>(c.rgb * t, c.a * t);
}
