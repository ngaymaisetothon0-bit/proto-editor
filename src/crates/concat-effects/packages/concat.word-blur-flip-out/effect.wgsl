struct Params { duration: f32 }

// easeOutCubic: 0 at x = 0, exactly 1 at x = 1, steep at the start
// and gentle at the landing.
fn ease_out_cubic(x: f32) -> f32 {
    return 1.0 - pow(1.0 - x, 3.0);
}
// Each word flips away like a card turning around its flat axis, while blurring away, last word first, over its last half-second.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    // Seconds left of the clip, or a very long time where the clip has no
    // known end - a treatment, a probe - so the look holds its rest there.
    let remain = select(1e9, frame.clip_duration - frame.clip_time, frame.clip_duration > 0.0);
    let p = clamp(remain / max(params.duration, 0.001), 0.0, 1.0);
    let e = 1.0 - p;
    // Reversed order: 0 for the last word, so it leaves first.
    let rr = 1.0 - reveal_order(uv);
    let v = 1.0 - clamp((e - rr * 0.75) * 4.0, 0.0, 1.0);
    // At rest (v = 1) the angle is 0, so cos is 1 and the squash is
    // the identity - the word sits at its rest shape.
    let angle = (1.0 - ease_out_cubic(v)) * 1.5707963;
    let co = cos(angle);
    let coord = vec2<f32>(uv.x, 0.5 + (uv.y - 0.5) / max(co, 0.08));
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
