struct Params { duration: f32 }

// easeOutCubic: 0 at x = 0, exactly 1 at x = 1, steep at the start
// and gentle at the landing.
fn ease_out_cubic(x: f32) -> f32 {
    return 1.0 - pow(1.0 - x, 3.0);
}
// Each word flips into place like a card turning around its flat axis, while sharpening out of a soft blur, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // The flip-blur-in formula: the word is squashed vertically around
    // its centre by 1 / cos(angle), edge-on at the start, face-on at
    // the end. The 0.08 floor is the reference's guard against dividing
    // by zero at the edge-on instant.
    let angle = (1.0 - ease_out_cubic(w)) * 1.5707963;
    let co = cos(angle);
    let coord = vec2<f32>(uv.x, 0.5 + (uv.y - 0.5) / max(co, 0.08));
    if (coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0) {
        return vec4<f32>(0.0);
    }
    // The same 3x3 box blur text-blur-in uses: at full progress the
    // radius is 0, so all nine taps read the same pixel and the
    // picture is exactly sharp.
    let r = texel() * (1.0 - w) * 8.0;
    var sum = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            sum += sample(coord + vec2<f32>(f32(x), f32(y)) * r);
        }
    }
    let c = sum / 9.0;
    return vec4<f32>(c.rgb * w, c.a * w);
}
