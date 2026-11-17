struct Params { duration: f32 }

// Rotate a point around the picture centre by `angle` radians,
// in pixels so a wide frame turns rigidly rather than shearing.
fn swing_coord(uv: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = (uv - vec2<f32>(0.5)) * frame.size;
    let s = sin(angle);
    let co = cos(angle);
    let r2 = vec2<f32>(c.x * co - c.y * s, c.x * s + c.y * co);
    return r2 / frame.size + vec2<f32>(0.5);
}
// Each word swings into place like a pendulum rocking around the picture centre, while sharpening out of a soft blur, one after another.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(frame.clip_time / max(params.duration, 0.001), 0.0, 1.0);
    let w = clamp((p - reveal_order(uv) * 0.75) * 4.0, 0.0, 1.0);
    // The swing-blur-in formula: a pendulum rock whose amplitude dies
    // out as the word arrives. At w = 1 the angle is 0, so the word
    // sits at its rest orientation.
    let angle = (1.0 - w) * 0.5 * sin(w * 9.42477796);
    let coord = swing_coord(uv, angle);
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
